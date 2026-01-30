use thiserror::Error;

/// Errors that can occur in repository operations.
#[derive(Error, Debug)]
pub enum RepositoryError {
    /// Generic database error
    #[error("Database error: {0}")]
    Database(String),

    /// Entity not found
    #[error("Not found")]
    NotFound,

    /// Duplicate entry (unique constraint violation)
    #[error("Duplicate entry: {0}")]
    Duplicate(String),

    /// Foreign key constraint violation
    #[error("Foreign key constraint violation: {0}")]
    ForeignKey(String),

    /// Invalid data provided
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Transaction failed
    #[error("Transaction failed: {0}")]
    Transaction(String),
}

/// Errors that can occur in service layer operations.
#[derive(Error, Debug)]
pub enum ServiceError {
    /// Repository layer error
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    /// Entity not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Conflict (e.g., duplicate entry)
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Bad request / validation error
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// Unauthorized access
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Forbidden action
    #[error("Forbidden: {0}")]
    Forbidden(String),

    /// External service error
    #[error("External service error: {0}")]
    ExternalService(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Errors that can occur in email service operations.
#[derive(Error, Debug)]
pub enum EmailError {
    /// Configuration error
    #[error("Email configuration error: {0}")]
    Configuration(String),

    /// Template rendering error
    #[error("Email template error: {0}")]
    Template(String),

    /// SMTP/sending error
    #[error("Email sending error: {0}")]
    Send(String),

    /// Invalid recipient
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
}
