
use std::fmt::Show;

use cursor;
use cursor::{Cursor};
use types::{BindArg, Null, Integer, Integer64, Float64, Text};
use types::{SQLITE_ROW, SQLITE_DONE};
use types::{SqliteResult, SQLITE_MISUSE};

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

impl ToSql for i64 {
    fn to_sql(&self) -> BindArg { Integer64(*self) }
}

impl ToSql for f64 {
    fn to_sql(&self) -> BindArg { Float64(*self) }
}

impl ToSql for Option<int> {
    fn to_sql(&self) -> BindArg {
        match *self {
            Some(i) => Integer(i),
            None => Null
        }
    }
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



#[cfg(test)]
mod query_tests {
    use database::Database;
    use types::SqliteResult;

    use std::to_string::ToString;

    #[test]
    fn bind2() {
        #[deriving(Show)]
        struct Person {
            id: int,
            name: String,
            address: String,
        }

        fn mk_person(id: int, name: &ToString, address: &ToString) -> Person {
            Person { id: id, name: name.to_string(), address: address.to_string() }
        }

        fn build() -> SqliteResult<Database> {
            let database = try!(Database::new(":memory:"));
            try!(database.exec(
                "CREATE TABLE test (id int, name text, address text)"));
            {
                let tx = try!(database.prepare(
                    "INSERT INTO test (id, name, address) VALUES (?, ?, ?)"));
                for who in [mk_person(1, &"John Doe", &"123 w Pine"),
                            mk_person(2, &"Jane Doe", &"345 e Walnut")].iter() {
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
}


#[cfg(test)]
mod api_tests {
    use database::Database;
    use types::{SqliteResult, Integer, Text};
    use types::{SQLITE_ROW, SQLITE_DONE};

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
