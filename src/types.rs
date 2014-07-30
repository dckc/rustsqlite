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

use std::collections::HashMap;
use std::num::from_uint;

#[deriving(PartialEq, Eq, Show)]
#[repr(C)]
pub enum ResultOk {
    SQLITE_OK = 0,
    _unused // avoid: unsupported representation for univariant enum [E0083]
}

#[deriving(PartialEq, Eq, Show, FromPrimitive)]
#[repr(C)]
pub enum ResultError {
    SQLITE_ERROR      =  1,
    SQLITE_INTERNAL   =  2,
    SQLITE_PERM       =  3,
    SQLITE_ABORT      =  4,
    SQLITE_BUSY       =  5,
    SQLITE_LOCKED     =  6,
    SQLITE_NOMEM      =  7,
    SQLITE_READONLY   =  8,
    SQLITE_INTERRUPT  =  9,
    SQLITE_IOERR      = 10,
    SQLITE_CORRUPT    = 11,
    SQLITE_NOTFOUND   = 12,
    SQLITE_FULL       = 13,
    SQLITE_CANTOPEN   = 14,
    SQLITE_PROTOCOL   = 15,
    SQLITE_EMPTY      = 16,
    SQLITE_SCHEMA     = 17,
    SQLITE_TOOBIG     = 18,
    SQLITE_CONSTRAINT = 19,
    SQLITE_MISMATCH   = 20,
    SQLITE_MISUSE     = 21,
    SQLITE_NOLFS      = 22,
    SQLITE_AUTH       = 23,
    SQLITE_FORMAT     = 24,
    SQLITE_RANGE      = 25,
    SQLITE_NOTADB     = 26,
}

#[deriving(PartialEq, Eq, Show, FromPrimitive)]
#[repr(C)]
pub enum ResultLog {
    SQLITE_NOTICE      = 27,
    SQLITE_WARNING    = 28,
}


#[deriving(PartialEq, Eq, Show, FromPrimitive)]
#[repr(C)]
pub enum ResultStep {
    SQLITE_ROW        = 100,
    SQLITE_DONE       = 101,
}

#[deriving(Show, PartialEq)]
pub enum BindArg {
    Text(String),
    StaticText(&'static str),
    Float64(f64),
    Integer(int),
    Integer64(i64),
    Blob(Vec<u8>),
    Null,
}

pub enum ColumnType {
    SQLITE_INTEGER,
    SQLITE_FLOAT,
    SQLITE_TEXT,
    SQLITE_BLOB,
    SQLITE_NULL,
}

#[must_use]
pub type SqliteResult<T> = Result<T, ResultError>;

// TODO: c_int?
pub fn check(r: i32) -> SqliteResult<()> {
    if r == SQLITE_OK as i32 { Ok(()) }
    else { Err(from_uint::<ResultError>(r as uint).unwrap()) }
}

pub type RowMap = HashMap<String, BindArg>;
