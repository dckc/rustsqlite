extern crate serialize;

use self::serialize::{Encoder, Encodable};

use cursor::Cursor;
use types::ResultCode;

/*
impl<'db> Encoder<ResultCode> for Cursor<'db> {
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
            let tx = try!(database.prepare(
                "INSERT INTO test (id, name, address) VALUES (?, ?, ?)"));
            for row in [(1i, "John Doe", "123 w Pine"),
                        (2i, "Jane Doe", "345 e Walnut")].iter() {
                (*row).encode(&tx);
                tx.step();
                tx.reset();
            }
            Ok(())
       }
    }
}
*/

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
