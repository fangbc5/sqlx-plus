use thiserror::Error;

#[derive(Debug, Error)]
pub enum SqlxPlusError {
    #[error("Unsupported database URL: {0}")]
    UnsupportedDatabase(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("No connection pool available for driver")]
    NoPoolAvailable,
    /// Transaction has already been consumed (committed or rolled back)
    #[error("Transaction has already been consumed")]
    AlreadyConsumed,
    /// Generic error message for compatibility
    #[error("{0}")]
    Other(String),
    /// Invalid field error
    #[error("Invalid field: {0}")]
    InvalidField(String),
    /// Not implemented error
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

pub type Result<T> = std::result::Result<T, SqlxPlusError>;