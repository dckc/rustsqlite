extern crate serialize;

use self::serialize::{Encoder, Encodable};

use cursor::{Cursor};
use types::{Null, Integer, Integer64, Float64, Text};
use types::ResultCode;
use types::{SqliteResult, SQLITE_OK, SQLITE_INTERNAL};

// TODO: push this into cursor, database
// rename ResultCode to ErrorCode and take out SQLITE_OK
fn res(code: ResultCode) -> SqliteResult<()> {
    match code {
        x if x == SQLITE_OK => Ok(()),
        err @ _ => Err(err)
    }
}

fn todo() -> SqliteResult<()> { Err(SQLITE_INTERNAL) }


impl<'db> Encoder<ResultCode> for Cursor<'db> {
    // add column state?
    fn emit_nil(&mut self) -> SqliteResult<()> { res(self.bind_param(0, &Null)) }

    fn emit_bool(&mut self, v: bool) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v as int))) }

    fn emit_int(&mut self, v: int) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v))) }
    // hmm... promote uint to i64 instead of int?
    fn emit_uint(&mut self, v: uint) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v as int))) }
    fn emit_i8(&mut self, v: i8) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v as int))) }
    fn emit_u8(&mut self, v: u8) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v as int))) }
    fn emit_i16(&mut self, v: i16) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v as int))) }
    fn emit_u16(&mut self, v: u16) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v as int))) }
    fn emit_i32(&mut self, v: i32) -> SqliteResult<()> { res(self.bind_param(0, &Integer(v as int))) }
    fn emit_u32(&mut self, v: u32) -> SqliteResult<()> { res(self.bind_param(0, &Integer64(v as i64))) }
    fn emit_i64(&mut self, v: i64) -> SqliteResult<()> { res(self.bind_param(0, &Integer64(v))) }
    // hmm... u64 as i64
    fn emit_u64(&mut self, v: u64) -> SqliteResult<()> { res(self.bind_param(0, &Integer64(v as i64))) }

    fn emit_f32(&mut self, v: f32) -> SqliteResult<()> { res(self.bind_param(0, &Float64(v as f64))) }
    fn emit_f64(&mut self, v: f64) -> SqliteResult<()> { res(self.bind_param(0, &Float64(v))) }

    fn emit_char(&mut self, v: char) -> SqliteResult<()> { res(self.bind_param(0, &Text(v.to_string()))) }
    fn emit_str(&mut self, v: &str) -> SqliteResult<()> { res(self.bind_param(0, &Text(v.to_string()))) }

    fn emit_enum(&mut self,
                 _name: &str,
                 f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }
    fn emit_enum_variant(&mut self,
                         v_name: &str, v_id: uint, len: uint,
                         f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        match len {
            0 => res(self.bind_param(0, &Text(v_name.to_string()))),
            _ => todo()
        }
    }
    fn emit_enum_variant_arg(&mut self,
                             a_idx: uint,
                             f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }

    fn emit_struct(&mut self,
                   name: &str, len: uint,
                   f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }

    fn emit_struct_field(&mut self,
                         f_name: &str, f_idx: uint,
                         f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo() // this one I think I actually want to do.
    }

    fn emit_enum_struct_variant(&mut self,
                                v_name: &str, v_id: uint, len: uint,
                                f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_enum_struct_variant_field(&mut self,
                                      f_name: &str, f_idx: uint,
                                      f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_tuple(&mut self, len: uint, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_tuple_arg(&mut self, idx: uint, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_tuple_struct(&mut self, name: &str, len: uint, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_tuple_struct_arg(&mut self, f_idx: uint, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_option(&mut self, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_option_none(&mut self) -> SqliteResult<()> {
        self.emit_nil()
    }

    fn emit_option_some(&mut self, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }
    fn emit_seq(&mut self, len: uint, f: |this: &mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_seq_elt(&mut self, idx: uint, f: |this: &mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        f(self)
    }
    fn emit_map(&mut self, len: uint, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_map_elt_key(&mut self, idx: uint, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
    fn emit_map_elt_val(&mut self, idx: uint, f: |&mut Cursor<'db>| -> SqliteResult<()>) -> SqliteResult<()> {
        todo()
    }
}


#[cfg(test)]
mod query_tests {
    use database::Database;
    use types::SqliteResult;
    use super::serialize::*;  // for Encodeable on tuples

    #[test]
    fn bind1() {
        fn go() -> SqliteResult<()> {
            let database = try!(Database::new(":memory:"));
             try!(database.exec(
                "CREATE TABLE test (id int, name text, address text)"));
            let mut tx = try!(database.prepare(
                "INSERT INTO test (id, name, address) VALUES (?, ?, ?)"));
            for row in [(1i, "John Doe", "123 w Pine"),
                        (2i, "Jane Doe", "345 e Walnut")].iter() {
                (*row).encode(&mut tx);
                tx.step();
                tx.reset();
            }
            Ok(())
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
            tx.bind_params([Integer(2), StaticText("Jane Doe"), StaticText("345 e Walnut")]);
            tx.step();

            let q = try!(database.prepare("select * from test"));
            q.step();
            let name = q.get_text(1);
            assert_eq!(name.as_slice(), "John Doe");

            q.step();
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
