/*
** Copyright (c) 2011, Brian Smith <brian@linuxfood.net>
** All rights reserved.
**
** Redistribution and use in source and binary forms, with or without
** modification, are permitted provided that the following conditions are met:
**
**   * Redistributions of source code must retain the above copyright notice,
**     this list of conditions and the following disclaimer.
**
**   * Redistributions in binary form must reproduce the above copyright notice,
**     this list of conditions and the following disclaimer in the documentation
**     and/or other materials provided with the distribution.
**
**   * Neither the name of Brian Smith nor the names of its contributors
**     may be used to endorse or promote products derived from this software
**     without specific prior written permission.
**
** THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
** AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
** IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
** ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
** LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
** CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
** SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
** INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
** CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
** ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
** POSSIBILITY OF SUCH DAMAGE.
*/

use cursor::*;
use ffi::*;
use libc::c_int;
use std::ptr;
use std::str;
use types::*;

/// The database connection.
pub struct Database {
    dbh: *mut dbh,
}

impl Drop for Database {
    /// Closes the database connection.
    /// See http://www.sqlite.org/c3ref/close.html
    fn drop(&mut self) {
        debug!("`Database.drop()`: dbh={:?}", self.dbh);
        unsafe {
            sqlite3_close(self.dbh);
        }
    }
}

impl Database {
    /// Opens a new database connection.
    /// `path` can either be a filesystem path or ":memory:".
    /// See http://www.sqlite.org/c3ref/open.html
    pub fn new(path: &str) -> SqliteResult<Database> {
        let mut dbh = ptr::mut_null();
        let r = path.with_c_str( |_path| {
            unsafe {
                sqlite3_open(_path, &mut dbh)
            }
        });
        match r {
            SQLITE_OK => {
                debug!("`open()`: dbh={:?}", dbh);
                Ok(Database { dbh: dbh })
            }
            _ => {
                unsafe {
                    sqlite3_close(dbh);
                }
                Err(r)
            }
        }
    }


    /// Returns the error message of the the most recent call.
    /// See http://www.sqlite.org/c3ref/errcode.html
    pub fn get_errmsg(&self) -> String {
        unsafe {
            str::raw::from_c_str(sqlite3_errmsg(self.dbh))
        }
    }

    /// Prepares/compiles an SQL statement.
    /// See http://www.sqlite.org/c3ref/prepare.html
    pub fn prepare<'db>(&'db self, sql: &str) -> SqliteResult<Cursor<'db>> {
        match self.prepare_with_offset(sql) {
            Ok((cur, _)) => Ok(cur),
            Err(e) => Err(e)
        }
    }
                
    pub fn prepare_with_offset<'db>(&'db self, sql: &str) -> SqliteResult<(Cursor<'db>, uint)> {
        let (r, stmt, offset) = sql.with_c_str( |_sql| {
            let mut new_stmt = ptr::mut_null();
            let mut tail = ptr::null();
            let r = unsafe {
                sqlite3_prepare_v2(self.dbh, _sql, sql.len() as c_int, &mut new_stmt, &mut tail)
            };
            (r, new_stmt, tail as uint - _sql as uint)
        });
        match r {
            SQLITE_OK => {
                debug!("`Database.prepare()`: stmt={:?}", stmt);
                Ok((Cursor::new(stmt, &self.dbh), offset))
            }
            _ => Err(r)
        }
    }

    /// Executes an SQL statement.
    /// See http://www.sqlite.org/c3ref/exec.html
    pub fn exec(&self, sql: &str) -> SqliteResult<()> {
        let mut r = SQLITE_ERROR;
        sql.with_c_str( |_sql| {
            unsafe {
                r = sqlite3_exec(self.dbh, _sql, ptr::mut_null(), ptr::mut_null(), ptr::mut_null())
            }
        });

        if r == SQLITE_OK { Ok(()) } else { Err(r) }
    }

    /// Returns the number of modified/inserted/deleted rows by the most recent
    /// call.
    /// See http://www.sqlite.org/c3ref/changes.html
    pub fn get_changes(&self) -> int {
        unsafe {
            sqlite3_changes(self.dbh) as int
        }
    }

    /// Returns the ID of the last inserted row.
    /// See http://www.sqlite.org/c3ref/last_insert_rowid.html
    pub fn get_last_insert_rowid(&self) -> i64 {
        unsafe {
            sqlite3_last_insert_rowid(self.dbh)
        }
    }

    /// Sets a busy timeout.
    /// See http://www.sqlite.org/c3ref/busy_timeout.html
    pub fn set_busy_timeout(&self, ms: int) -> ResultCode {
        unsafe {
            sqlite3_busy_timeout(self.dbh, ms as c_int)
        }
    }
}
