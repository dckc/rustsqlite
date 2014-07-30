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

#![allow(non_camel_case_types)]

use libc::*;

pub enum dbh {}
pub enum stmt {}
pub enum _notused {}

#[link(name = "sqlite3")]
extern {
    pub fn sqlite3_open(path: *const c_char, hnd: *mut *mut dbh) -> c_int;
    pub fn sqlite3_close_v2(dbh: *mut dbh) -> c_int;
    pub fn sqlite3_errmsg(dbh: *mut dbh) -> *const c_char;
    pub fn sqlite3_changes(dbh: *mut dbh) -> c_int;
    pub fn sqlite3_last_insert_rowid(dbh: *mut dbh) -> i64;
    pub fn sqlite3_complete(sql: *const c_char) -> c_int;

    pub fn sqlite3_prepare_v2(
        hnd: *mut dbh,
        sql: *const c_char,
        sql_len: c_int,
        shnd: *mut *mut stmt,
        tail: *mut *const c_char
    ) -> c_int;

    pub fn sqlite3_exec(
        dbh: *mut dbh,
        sql: *const c_char,
        cb: *mut _notused,
        d: *mut _notused,
        err: *mut *mut c_char
    ) -> c_int;

    pub fn sqlite3_step(sth: *mut stmt) -> c_int;
    pub fn sqlite3_reset(sth: *mut stmt) -> c_int;
    pub fn sqlite3_finalize(sth: *mut stmt) -> c_int;
    pub fn sqlite3_clear_bindings(sth: *mut stmt) -> c_int;

    pub fn sqlite3_column_name(sth: *mut stmt, icol: c_int) -> *const c_char;
    pub fn sqlite3_column_type(sth: *mut stmt, icol: c_int) -> c_int;
    pub fn sqlite3_data_count(sth: *mut stmt) -> c_int;
    pub fn sqlite3_column_bytes(sth: *mut stmt, icol: c_int) -> c_int;
    pub fn sqlite3_column_blob(sth: *mut stmt, icol: c_int) -> *const u8;

    pub fn sqlite3_column_text(sth: *mut stmt, icol: c_int) -> *const c_char;
    pub fn sqlite3_column_double(sth: *mut stmt, icol: c_int) -> f64;
    pub fn sqlite3_column_int(sth: *mut stmt, icol: c_int) -> c_int;
    pub fn sqlite3_column_int64(sth: *mut stmt, icol: c_int) -> i64;

    pub fn sqlite3_bind_blob(sth: *mut stmt, icol: c_int, buf: *const u8, buflen: c_int, d: *mut c_void) -> c_int;
    pub fn sqlite3_bind_text(sth: *mut stmt, icol: c_int, buf: *const c_char, buflen: c_int, d: *mut c_void) -> c_int;
    pub fn sqlite3_bind_null(sth: *mut stmt, icol: c_int) -> c_int;
    pub fn sqlite3_bind_int(sth: *mut stmt, icol: c_int, v: c_int) -> c_int;
    pub fn sqlite3_bind_int64(sth: *mut stmt, icol: c_int, v: i64) -> c_int;
    pub fn sqlite3_bind_double(sth: *mut stmt, icol: c_int, value: f64) -> c_int;
    pub fn sqlite3_bind_parameter_index(sth: *mut stmt, name: *const c_char) -> c_int;

    pub fn sqlite3_busy_timeout(dbh: *mut dbh, ms: c_int) -> c_int;
}
