// Copyright 2023 Ross Light
// SPDX-License-Identifier: MIT

use std::ffi::{c_int, CStr};
use std::fmt;
use std::ptr::NonNull;

use libsqlite3_sys::{
    sqlite3, sqlite3_errmsg, sqlite3_errstr, sqlite3_extended_errcode, SQLITE_ABORT, SQLITE_AUTH,
    SQLITE_BUSY, SQLITE_CANTOPEN, SQLITE_CONSTRAINT, SQLITE_CORRUPT, SQLITE_DONE, SQLITE_EMPTY,
    SQLITE_ERROR, SQLITE_FORMAT, SQLITE_FULL, SQLITE_INTERNAL, SQLITE_INTERRUPT, SQLITE_IOERR,
    SQLITE_LOCKED, SQLITE_MISMATCH, SQLITE_MISUSE, SQLITE_NOLFS, SQLITE_NOMEM, SQLITE_NOTADB,
    SQLITE_NOTFOUND, SQLITE_NOTICE, SQLITE_OK, SQLITE_PERM, SQLITE_PROTOCOL, SQLITE_RANGE,
    SQLITE_READONLY, SQLITE_ROW, SQLITE_SCHEMA, SQLITE_TOOBIG, SQLITE_WARNING,
};

/// The numeric [result code] of a SQLite function.
///
/// [result code]: https://www.sqlite.org/rescode.html
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResultCode(pub(crate) c_int);

impl ResultCode {
    /// Indicates that the operation was successful and there were no errors.
    pub const OK: ResultCode = ResultCode(SQLITE_OK);
    /// Indicates that another row of output is available.
    pub const ROW: ResultCode = ResultCode(SQLITE_ROW);
    /// Indicates that an operation has completed.
    pub const DONE: ResultCode = ResultCode(SQLITE_DONE);

    /// A generic error code that is used
    /// when no other more specific error code is available.
    pub const ERROR: ResultCode = ResultCode(SQLITE_ERROR);
    /// Indicates an internal malfunction.
    /// In a working version of SQLite,
    /// an application should never see this result code.
    pub const INTERNAL: ResultCode = ResultCode(SQLITE_INTERNAL);
    /// Indicates that the requested access mode for a newly created database
    /// could not be provided.
    pub const PERM: ResultCode = ResultCode(SQLITE_PERM);
    /// Indicates that an operation was aborted prior to completion,
    /// usually be application request.
    pub const ABORT: ResultCode = ResultCode(SQLITE_ABORT);
    /// Indicates that the database file could not be written (or in some cases read)
    /// because of concurrent activity by some other database connection,
    /// usually a database connection in a separate process.
    pub const BUSY: ResultCode = ResultCode(SQLITE_BUSY);
    /// Indicates that a write operation could not continue
    /// because of a conflict within the same database connection
    /// or a conflict with a different database connection that uses a shared cache.
    pub const LOCKED: ResultCode = ResultCode(SQLITE_LOCKED);
    /// Indicates that SQLite was unable to allocate all the memory it needed
    /// to complete the operation.
    pub const NOMEM: ResultCode = ResultCode(SQLITE_NOMEM);
    /// Indicates an attempt was made to alter some data
    /// for which the current database connection does not have write permission.
    pub const READONLY: ResultCode = ResultCode(SQLITE_READONLY);
    /// Indicates that an operation was interrupted by the `sqlite3_interrupt()` interface.
    pub const INTERRUPT: ResultCode = ResultCode(SQLITE_INTERRUPT);
    /// Indicates that the operation could not finish
    /// because the operating system reported an I/O error.
    pub const IOERR: ResultCode = ResultCode(SQLITE_IOERR);
    /// Indicates that the database file has been corrupted.
    pub const CORRUPT: ResultCode = ResultCode(SQLITE_CORRUPT);
    pub const NOTFOUND: ResultCode = ResultCode(SQLITE_NOTFOUND);
    /// Indicates that a write could not complete because the disk is full.
    pub const FULL: ResultCode = ResultCode(SQLITE_FULL);
    /// Indicates that SQLite was unable to open a file.
    pub const CANTOPEN: ResultCode = ResultCode(SQLITE_CANTOPEN);
    /// Indicates a problem with the file locking protocol used by SQLite.
    pub const PROTOCOL: ResultCode = ResultCode(SQLITE_PROTOCOL);
    /// Not currently used.
    pub const EMPTY: ResultCode = ResultCode(SQLITE_EMPTY);
    /// Indicates that the database schema has changed.
    pub const SCHEMA: ResultCode = ResultCode(SQLITE_SCHEMA);
    /// Indicates that a string or `BLOB` was too large.
    pub const TOOBIG: ResultCode = ResultCode(SQLITE_TOOBIG);
    /// Indicates that an SQL constraint violation occurred
    /// while trying to process an SQL statement.
    pub const CONSTRAINT: ResultCode = ResultCode(SQLITE_CONSTRAINT);
    /// Indicates a datatype mismatch.
    pub const MISMATCH: ResultCode = ResultCode(SQLITE_MISMATCH);
    /// Indicates that the application used a SQLite interface
    /// in a way that is undefined or unsupported.
    pub const MISUSE: ResultCode = ResultCode(SQLITE_MISUSE);
    /// Can be returned on systems that do not support large files
    /// when the database grows to be larger than what the filesystem can handle.
    /// `NOLFS` stands for "NO Large File Support".
    pub const NOLFS: ResultCode = ResultCode(SQLITE_NOLFS);
    /// Returned when the authorizer callback
    /// indicates that an SQL statement being prepared is not authorized.
    pub const AUTH: ResultCode = ResultCode(SQLITE_AUTH);
    /// Not currently used.
    pub const FORMAT: ResultCode = ResultCode(SQLITE_FORMAT);
    /// Indicates that
    /// the parameter number argument to one of the `sqlite3_bind` routines
    /// or the column number in one of the `sqlite3_column` routines
    /// is out of range.
    pub const RANGE: ResultCode = ResultCode(SQLITE_RANGE);
    /// Indicates that the file being opened
    /// does not appear to be an SQLite database file.
    pub const NOTADB: ResultCode = ResultCode(SQLITE_NOTADB);
    /// Used in log callbacks
    /// to indicate that an unusual operation is taking place.
    pub const NOTICE: ResultCode = ResultCode(SQLITE_NOTICE);
    /// Used in log callbacks
    /// to indicate that an unusual and possibly ill-advised operation is taking place.
    pub const WARNING: ResultCode = ResultCode(SQLITE_WARNING);

    /// Converts the result code to a [primary result code],
    /// which is a category of errors.
    ///
    /// [primary result code]: https://www.sqlite.org/rescode.html#primary_result_codes_versus_extended_result_codes
    #[inline]
    pub const fn to_primary(self) -> ResultCode {
        ResultCode(self.0 & 0xff)
    }

    /// Reports whether the primary result code is one of
    /// [`OK`][ResultCode::OK],
    /// [`ROW`][ResultCode::ROW],
    /// or [`DONE`][ResultCode::DONE].
    #[inline]
    pub const fn is_success(self) -> bool {
        match self.to_primary() {
            ResultCode::OK | ResultCode::ROW | ResultCode::DONE => true,
            _ => false,
        }
    }

    /// Returns the English-language text that describes the result code.
    pub fn message(self) -> &'static str {
        let s = unsafe { CStr::from_ptr(sqlite3_errstr(self.0)) };
        s.to_str().unwrap_or("unknown error")
    }

    /// Converts a result code to a [`Result`].
    /// Successful codes will be a `Ok` of the code itself
    /// and unsuccessful codes will be converted into an [`Error`].
    pub fn to_result(self) -> Result<ResultCode> {
        if self.is_success() {
            Ok(self)
        } else {
            Err(Error {
                result_code: self,
                msg: String::new(),
                error_offset: None,
            })
        }
    }
}

impl From<ResultCode> for Result<ResultCode> {
    fn from(rc: ResultCode) -> Self {
        rc.to_result()
    }
}

impl From<ResultCode> for c_int {
    fn from(rc: ResultCode) -> Self {
        rc.0
    }
}

impl Default for ResultCode {
    fn default() -> Self {
        ResultCode::OK
    }
}

impl fmt::Debug for ResultCode {
    /// Writes the SQLite C constant name, if known,
    /// or the default debug formatting otherwise.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! format_constant_names {
            ( $x:expr, $f:expr, $($i:ident),* $(,)* ) => {
                {
                    let x = $x;
                    match x {
                        $(
                            ::libsqlite3_sys::$i => Some($f.write_str(::std::stringify!($i))),
                        )*
                        _ => None,
                    }
                }
            };
        }

        #[cfg(any(feature = "modern", feature = "buildtime_bindgen"))]
        {
            let maybe_result = format_constant_names!(
                self.0,
                f,
                SQLITE_ERROR_MISSING_COLLSEQ,
                SQLITE_ERROR_RETRY,
                SQLITE_ERROR_SNAPSHOT,
                SQLITE_IOERR_BEGIN_ATOMIC,
                SQLITE_IOERR_COMMIT_ATOMIC,
                SQLITE_IOERR_ROLLBACK_ATOMIC,
                SQLITE_IOERR_DATA,
                SQLITE_IOERR_CORRUPTFS,
                SQLITE_LOCKED_VTAB,
                SQLITE_BUSY_TIMEOUT,
                SQLITE_CANTOPEN_DIRTYWAL,
                SQLITE_CANTOPEN_SYMLINK,
                SQLITE_CORRUPT_SEQUENCE,
                SQLITE_CORRUPT_INDEX,
                SQLITE_READONLY_CANTINIT,
                SQLITE_READONLY_DIRECTORY,
                SQLITE_CONSTRAINT_PINNED,
                SQLITE_CONSTRAINT_DATATYPE,
                SQLITE_NOTICE_RBU,
                SQLITE_OK_SYMLINK,
            );
            if let Some(result) = maybe_result {
                return result;
            }
        }

        let maybe_result = format_constant_names!(
            self.0,
            f,
            SQLITE_OK,
            SQLITE_ERROR,
            SQLITE_INTERNAL,
            SQLITE_PERM,
            SQLITE_ABORT,
            SQLITE_BUSY,
            SQLITE_LOCKED,
            SQLITE_NOMEM,
            SQLITE_READONLY,
            SQLITE_INTERRUPT,
            SQLITE_IOERR,
            SQLITE_CORRUPT,
            SQLITE_NOTFOUND,
            SQLITE_FULL,
            SQLITE_CANTOPEN,
            SQLITE_PROTOCOL,
            SQLITE_EMPTY,
            SQLITE_SCHEMA,
            SQLITE_TOOBIG,
            SQLITE_CONSTRAINT,
            SQLITE_MISMATCH,
            SQLITE_MISUSE,
            SQLITE_NOLFS,
            SQLITE_AUTH,
            SQLITE_FORMAT,
            SQLITE_RANGE,
            SQLITE_NOTADB,
            SQLITE_NOTICE,
            SQLITE_WARNING,
            SQLITE_ROW,
            SQLITE_DONE,
            SQLITE_IOERR_READ,
            SQLITE_IOERR_SHORT_READ,
            SQLITE_IOERR_WRITE,
            SQLITE_IOERR_FSYNC,
            SQLITE_IOERR_DIR_FSYNC,
            SQLITE_IOERR_TRUNCATE,
            SQLITE_IOERR_FSTAT,
            SQLITE_IOERR_UNLOCK,
            SQLITE_IOERR_RDLOCK,
            SQLITE_IOERR_DELETE,
            SQLITE_IOERR_BLOCKED,
            SQLITE_IOERR_NOMEM,
            SQLITE_IOERR_ACCESS,
            SQLITE_IOERR_CHECKRESERVEDLOCK,
            SQLITE_IOERR_LOCK,
            SQLITE_IOERR_CLOSE,
            SQLITE_IOERR_DIR_CLOSE,
            SQLITE_IOERR_SHMOPEN,
            SQLITE_IOERR_SHMSIZE,
            SQLITE_IOERR_SHMLOCK,
            SQLITE_IOERR_SHMMAP,
            SQLITE_IOERR_SEEK,
            SQLITE_IOERR_DELETE_NOENT,
            SQLITE_IOERR_MMAP,
            SQLITE_IOERR_GETTEMPPATH,
            SQLITE_IOERR_CONVPATH,
            SQLITE_IOERR_VNODE,
            SQLITE_IOERR_AUTH,
            SQLITE_LOCKED_SHAREDCACHE,
            SQLITE_BUSY_RECOVERY,
            SQLITE_BUSY_SNAPSHOT,
            SQLITE_CANTOPEN_NOTEMPDIR,
            SQLITE_CANTOPEN_ISDIR,
            SQLITE_CANTOPEN_FULLPATH,
            SQLITE_CANTOPEN_CONVPATH,
            SQLITE_CORRUPT_VTAB,
            SQLITE_READONLY_RECOVERY,
            SQLITE_READONLY_CANTLOCK,
            SQLITE_READONLY_ROLLBACK,
            SQLITE_READONLY_DBMOVED,
            SQLITE_ABORT_ROLLBACK,
            SQLITE_CONSTRAINT_CHECK,
            SQLITE_CONSTRAINT_COMMITHOOK,
            SQLITE_CONSTRAINT_FOREIGNKEY,
            SQLITE_CONSTRAINT_FUNCTION,
            SQLITE_CONSTRAINT_NOTNULL,
            SQLITE_CONSTRAINT_PRIMARYKEY,
            SQLITE_CONSTRAINT_TRIGGER,
            SQLITE_CONSTRAINT_UNIQUE,
            SQLITE_CONSTRAINT_VTAB,
            SQLITE_CONSTRAINT_ROWID,
            SQLITE_NOTICE_RECOVER_WAL,
            SQLITE_NOTICE_RECOVER_ROLLBACK,
            SQLITE_WARNING_AUTOINDEX,
            SQLITE_AUTH_USER,
            SQLITE_OK_LOAD_PERMANENTLY,
        );

        match maybe_result {
            Some(result) => result,
            None => f.debug_tuple("ResultCode").field(&self.0).finish(),
        }
    }
}

impl fmt::Display for ResultCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

/// A `Result` with a SQLite error.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// SQLite error.
#[derive(Clone, Debug)]
pub struct Error {
    result_code: ResultCode,
    msg: String,
    error_offset: Option<usize>,
}

impl Error {
    /// Returns a new `Error` with the given error code and message.
    /// If the message is empty,
    /// then the result code's English-language text will be used.
    ///
    /// # Panics
    ///
    /// Panics if `result_code` represents success.
    pub fn new(result_code: ResultCode, msg: impl Into<String>) -> Error {
        assert!(!result_code.is_success());
        Error {
            result_code,
            msg: msg.into(),
            error_offset: None,
        }
    }

    /// Extracts error information from the connection.
    pub(crate) fn get(db: NonNull<sqlite3>) -> Option<Self> {
        let result_code = ResultCode(unsafe { sqlite3_extended_errcode(db.as_ptr()) });
        if result_code.is_success() {
            return None;
        }
        let error_offset = Self::get_error_offset(db);
        let msg: &CStr = unsafe { CStr::from_ptr(sqlite3_errmsg(db.as_ptr())) };
        let msg = String::from_utf8_lossy(msg.to_bytes()).into_owned();
        Some(Error {
            result_code,
            msg,
            error_offset,
        })
    }

    fn get_error_offset(db: NonNull<sqlite3>) -> Option<usize> {
        #[cfg(feature = "modern")]
        {
            let error_offset = unsafe { libsqlite3_sys::sqlite3_error_offset(db.as_ptr()) };
            if error_offset < 0 {
                None
            } else {
                Some(error_offset as usize)
            }
        }

        #[cfg(not(feature = "modern"))]
        {
            let _ = db;
            None
        }
    }

    /// Returns the result code that produced this error.
    /// Guaranteed to not be a successful result code.
    #[inline]
    pub fn result_code(&self) -> ResultCode {
        self.result_code
    }

    /// Returns the byte offset of the start of the token that caused the error,
    /// if relevant.
    #[inline]
    pub fn error_offset(&self) -> Option<usize> {
        self.error_offset
    }

    /// Sets the error offset to `None`.
    /// Useful for masking the error offset when returning the error up the call stack.
    #[inline]
    pub fn clear_error_offset(&mut self) {
        self.error_offset = None;
    }

    /// Returns the error's message.
    /// Guaranteed to not be empty.
    pub fn message(&self) -> &str {
        if self.msg.is_empty() {
            self.result_code.message()
        } else {
            &self.msg
        }
    }
}

impl From<&Error> for ResultCode {
    fn from(err: &Error) -> Self {
        err.result_code()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.msg.is_empty() {
            self.result_code.fmt(f)
        } else {
            f.write_str(&self.msg)
        }
    }
}
