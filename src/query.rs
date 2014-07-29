extern crate serialize;

use std::fmt::Show;

use self::serialize::{Encoder};

use cursor;
use cursor::{Cursor};
use types::{BindArg, Null, Integer, Integer64, Float64, Text};
use types::{SQLITE_ROW, SQLITE_DONE};
use types::{ResultError, SqliteResult, SQLITE_MISUSE};

fn todo() -> SqliteResult<()> { fail!("TODO") }


// inspired by http://www.rust-ci.org/sfackler/rust-postgres/doc/postgres/trait.RowIndex.html
pub trait RowIndex {
    fn idx(&self, cursor: &Cursor) -> Option<uint>;
}

impl RowIndex for uint {
    fn idx(&self, _cursor: &Cursor) -> Option<uint> { Some(*self) }
}

impl RowIndex for &'static str {
    fn idx(&self, cursor: &Cursor) -> Option<uint> {
        let mut ixs = range(0, cursor.get_column_count() as uint);
        ixs.find(|ix| cursor.get_column_name(*ix as int).as_slice() == *self)
    }
}

trait FromSql {
    // col is provided in case you want to get the sqlite type of that col
    fn from_sql(cursor: &Cursor, col: uint) -> SqliteResult<Self>;
}

impl FromSql for int {
    // TODO: get_int should take a uint, not an int, right?
    fn from_sql(cursor: &Cursor, col: uint) -> SqliteResult<int> { Ok(cursor.get_int(col as int)) }
}

impl FromSql for String {
    fn from_sql(cursor: &Cursor, col: uint) -> SqliteResult<String> { Ok(cursor.get_text(col as int)) }
}

pub trait ToSql {
    fn to_sql(&self) -> BindArg;
}

impl ToSql for int {
    fn to_sql(&self) -> BindArg { Integer(*self) }
}

impl ToSql for String {
    // TODO: eliminate copy?
    fn to_sql(&self) -> BindArg { Text(self.clone()) }
}

pub struct Rows<'c> {
    cursor: &'c Cursor<'c>
}


impl<'c> Iterator<SqliteResult<()>> for Rows<'c> {
    fn next(&mut self) -> Option<SqliteResult<()>> {
        match self.cursor.step() {
            Ok(SQLITE_DONE) => None,
            Ok(SQLITE_ROW) => Some(Ok(())),
            Err(err) => Some(Err(err))
        }
    }
}


impl<'c> cursor::Cursor<'c> {
    // TODO: bind by name?
    fn query(&self, params: &[&ToSql]) -> SqliteResult<Rows> {
        try!(self.reset()); // need this?
        for (ix, param) in params.iter().enumerate() {
            try!(self.bind_param(ix + 1, &param.to_sql()))
        }
        Ok(Rows { cursor: self })
    }

    // TODO: bind by name?
    fn execute(&self, params: &[&ToSql]) -> SqliteResult<uint> {
        // TODO: factor out stuff in common with query()
        try!(self.reset()); // need this?
        for (ix, param) in params.iter().enumerate() {
            try!(self.bind_param(ix + 1, &param.to_sql()))
        }
        match self.step() {
            Ok(SQLITE_ROW) => Ok(0),
            Ok(SQLITE_DONE) => Ok(0), //@@ oops! we need the db to do .get_changes()
            Err(err) => Err(err)
        }
    }

    pub fn get_opt<I: RowIndex, T: FromSql>(&self, idx: I) -> SqliteResult<T> {
        match idx.idx(self) {
            Some(idx) => FromSql::from_sql(self, idx),
            None => Err(SQLITE_MISUSE)
        }
    }

    pub fn get<I: RowIndex + Show + Clone, T: FromSql>(&self, idx: I) -> T {
        match self.get_opt(idx.clone()) {
            Ok(ok) => ok,
            Err(err) => fail!("error retrieving column {}: {}", idx, err)
        }
    }
}

struct BindEncoder<'db> {
    cursor: &'db Cursor<'db>,
    col: uint
}


impl<'db> BindEncoder<'db> {
    pub fn new(cursor: &'db Cursor) -> BindEncoder<'db> {
        BindEncoder{ cursor: cursor, col: 0 }
    }

    fn push(&mut self, arg: &BindArg) -> SqliteResult<()> {
        if self.col == 0 { try!(self.cursor.reset()) }
        self.col += 1;
        self.cursor.bind_param(self.col, arg)
    }
}

impl<'db> Encoder<ResultError> for BindEncoder<'db> {
    // add column state?
    fn emit_nil(&mut self) -> SqliteResult<()> { self.push(&Null) }

    fn emit_bool(&mut self, v: bool) -> SqliteResult<()> { self.push(&Integer(v as int)) }

    fn emit_int(&mut self, v: int) -> SqliteResult<()> { self.push(&Integer(v)) }
    // hmm... promote uint to i64 instead of int?
    fn emit_uint(&mut self, v: uint) -> SqliteResult<()> { self.push(&Integer(v as int)) }
    fn emit_i8(&mut self, v: i8) -> SqliteResult<()> { self.push(&Integer(v as int)) }
    fn emit_u8(&mut self, v: u8) -> SqliteResult<()> { self.push(&Integer(v as int)) }
    fn emit_i16(&mut self, v: i16) -> SqliteResult<()> { self.push(&Integer(v as int)) }
    fn emit_u16(&mut self, v: u16) -> SqliteResult<()> { self.push(&Integer(v as int)) }
    fn emit_i32(&mut self, v: i32) -> SqliteResult<()> { self.push(&Integer(v as int)) }
    fn emit_u32(&mut self, v: u32) -> SqliteResult<()> { self.push(&Integer64(v as i64)) }
    fn emit_i64(&mut self, v: i64) -> SqliteResult<()> { self.push(&Integer64(v)) }
    // hmm... u64 as i64
    fn emit_u64(&mut self, v: u64) -> SqliteResult<()> { self.push(&Integer64(v as i64)) }

    fn emit_f32(&mut self, v: f32) -> SqliteResult<()> { self.push(&Float64(v as f64)) }
    fn emit_f64(&mut self, v: f64) -> SqliteResult<()> { self.push(&Float64(v)) }

    fn emit_char(&mut self, v: char) -> SqliteResult<()> { self.push(&Text(v.to_string())) }
    // extra copy?
    fn emit_str(&mut self, v: &str) -> SqliteResult<()> { self.push(&Text(v.to_string())) }

    fn emit_enum(&mut self,
                 _name: &str,
                 f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }
    fn emit_enum_variant(&mut self,
                         v_name: &str, _v_id: uint, len: uint,
                         _f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        match len {
            0 => self.push(&Text(v_name.to_string())),
            _ => todo()
        }
    }
    fn emit_enum_variant_arg(&mut self,
                             a_idx: uint,
                             f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }

    fn emit_struct(&mut self,
                   name: &str, len: uint,
                   f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }

    fn emit_struct_field(&mut self,
                         f_name: &str, f_idx: uint,
                         f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo() // this one I think I actually want to do.
    }

    fn emit_enum_struct_variant(&mut self,
                                v_name: &str, v_id: uint, len: uint,
                                f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_enum_struct_variant_field(&mut self,
                                      f_name: &str, f_idx: uint,
                                      f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_tuple(&mut self, len: uint, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        self.col = 0;
        f(self)
    }
    fn emit_tuple_arg(&mut self, idx: uint, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_tuple_struct(&mut self, name: &str, len: uint, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_tuple_struct_arg(&mut self, f_idx: uint, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_option(&mut self, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_option_none(&mut self) -> SqliteResult<()> {
        self.emit_nil()
    }

    fn emit_option_some(&mut self, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }
    fn emit_seq(&mut self, len: uint, f: |this: &mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_seq_elt(&mut self, idx: uint, f: |this: &mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }
    fn emit_map(&mut self, len: uint, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_map_elt_key(&mut self, idx: uint, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_map_elt_val(&mut self, idx: uint, f: |&mut BindEncoder<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
}


#[cfg(test)]
mod query_tests {
    use database::Database;
    use types::SqliteResult;
    use super::serialize::*;  // for Encodeable on tuples
    use super::BindEncoder;
    use types::{SQLITE_ROW, SQLITE_DONE};

    use std::to_string::ToString;

    #[test]
    fn bind2() {
        #[deriving(Show)]
        struct Person {
            id: int,
            name: String,
            address: String,
        }

        fn mkPerson(id: int, name: &ToString, address: &ToString) -> Person {
            Person { id: id, name: name.to_string(), address: address.to_string() }
        }

        fn build() -> SqliteResult<Database> {
            let database = try!(Database::new(":memory:"));
            try!(database.exec(
                "CREATE TABLE test (id int, name text, address text)"));
            {
                let tx = try!(database.prepare(
                    "INSERT INTO test (id, name, address) VALUES (?, ?, ?)"));
                for who in [mkPerson(1, &"John Doe", &"123 w Pine"),
                            mkPerson(2, &"Jane Doe", &"345 e Walnut")].iter() {
                    try!(tx.execute([&who.id, &who.name, &who.address])); // ignore number of rows
                }
            }
            Ok(database)
        }

        fn show(database: &Database) -> SqliteResult<String> {
            let q = try!(database.prepare("select id, name as name, address as address from test"));
            let persons = try!(q.query([])).map(
                // TODO: something with non-Ok results
                |result| Person { id: q.get(0u), name: q.get("name"), address: q.get("address") }
                );
            Ok(persons.map(|who| who.to_string()).collect::<Vec<String>>().connect("\n"))
        }
        match build() {
            Ok(db) => match show(&db) {
                // TODO: check the result by machine, not by eyeballing it
                Ok(txt) => debug!("=== DB:\n{}", txt),
                Err(oops) => fail!("show() Err: {}", oops)
            },
            Err(oops) => fail!("build() Err: {}", oops)
        }
    }

    #[test]
    fn bind1() {
        fn build() -> SqliteResult<Database> {
            let database = try!(Database::new(":memory:"));
            try!(database.exec(
                "CREATE TABLE test (id int, name text, address text)"));
            {
                let tx = try!(database.prepare(
                    "INSERT INTO test (id, name, address) VALUES (?, ?, ?)"));
                let mut b = BindEncoder::new(&tx);
                for row in [(1i, "John Doe", "123 w Pine"),
                            (2i, "Jane Doe", "345 e Walnut")].iter() {
                    try!((*row).encode(&mut b));
                    try!(tx.step());
                    try!(tx.reset());
                }
            }
            Ok(database)
        }

        fn show(database: &Database) -> SqliteResult<String> {
            let q = try!(database.prepare("select * from test"));
            let mut out = "".to_string();
            loop {
                match q.step() {
                    Ok(SQLITE_DONE) => break,
                    Ok(SQLITE_ROW) => {
                        for col in range(0, q.get_column_count()) {
                            debug!("col {:?} type: {:?}", col, q.get_column_type(col));
                            // hmm... is get_text() safe?
                            let cell = q.get_text(col);
                            debug!("col {:?}: {}", col, cell);
                            out.push_str(cell.as_slice());
                            out.push_char('|');
                        }
                        out.push_char('\n')
                    }
                    Err(err) => return Err(err)
                }
            }
            Ok(out)
        }
        match build() {
            Ok(db) => match show(&db) {
                Ok(txt) => debug!("=== DB:\n{}", txt),
                Err(oops) => fail!("show() Err: {}", oops)
            },
            Err(oops) => fail!("build() Err: {}", oops)
        }
    }
}


#[cfg(test)]
mod api_tests {
    use database::Database;
    use types::SqliteResult;
    use types::*; // TODO: I just want BindArg

    #[test]
    fn bind_fun() {
        fn go() -> SqliteResult<()> {
            let database = try!(Database::new(":memory:"));

            try!(database.exec(
                "BEGIN;
                CREATE TABLE test (id int, name text, address text);
                INSERT INTO test (id, name, address) VALUES (1, 'John Doe', '123 w Pine');
                COMMIT;"));

            let tx = try!(database.prepare(
                "INSERT INTO test (id, name, address) VALUES (?, ?, ?)"));
            try!(tx.bind_params([Integer(2), Text("Jane Doe".to_string()), Text("345 e Walnut".to_string())]));
            assert_eq!(tx.step(), Ok(SQLITE_DONE));
            assert_eq!(database.get_changes(), 1);

            let q = try!(database.prepare("select * from test order by id"));
            assert_eq!(q.step(), Ok(SQLITE_ROW));
            assert_eq!(q.get_int(0), 1);
            let name = q.get_text(1);
            assert_eq!(name.as_slice(), "John Doe");

            assert_eq!(q.step(), Ok(SQLITE_ROW));
            assert_eq!(q.get_int(0), 2);
            let addr = q.get_text(2);
            assert_eq!(addr.as_slice(), "345 e Walnut");
            Ok(())
        }
        match go() {
            Ok(_) => (),
            Err(e) => fail!("oops! {}", e)
        }
    }
}
