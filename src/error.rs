use thiserror::Error;

#[derive(Debug, Error)]
pub enum KvError {
    #[error("key not found: {0}")]
    KeyNotFound(String),

    #[error("database already exists: {0}")]
    DatabaseExists(String),

    #[error("database does not exist: {0}")]
    DatabaseMissing(String),

    #[error("database file is invalid, corrupt, or unsupported")]
    InvalidDatabase,

    #[error("could not unlock database; password may be incorrect or the file may be damaged")]
    DatabaseUnlockFailed,

    #[error("password confirmation did not match")]
    PasswordMismatch,

    #[error("password must not be empty")]
    EmptyPassword,

    #[error("database schema version {0} is not supported")]
    UnsupportedSchema(i64),

    #[error("stored value is not valid UTF-8 text")]
    InvalidUtf8,

    #[error("database path is not a regular file: {0}")]
    NotAFile(String),

    #[error("database operation failed: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("filesystem operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not determine a user data directory")]
    NoDataDirectory,

    #[error("password prompt failed: {0}")]
    PasswordPrompt(String),
}

pub type Result<T> = std::result::Result<T, KvError>;
