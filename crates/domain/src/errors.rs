use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Not found")]
    NotFound,

    #[error("Duplicate entry: {0}")]
    Duplicate(String),
}
