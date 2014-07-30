extern crate serialize;

use self::serialize::{Encoder};

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

fn todo() -> SqliteResult<()> { fail!("TODO") }


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
mod tests {
    use database::Database;
    use types::SqliteResult;
    use super::serialize::*;  // for Encodeable on tuples
    use super::BindEncoder;
    use types::{SQLITE_ROW, SQLITE_DONE};

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
