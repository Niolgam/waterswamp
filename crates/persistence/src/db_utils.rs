//! Database utility functions for error mapping and common operations.
//!
//! This module provides shared utilities to avoid code duplication across repositories.

use domain::errors::RepositoryError;

/// PostgreSQL error codes for constraint violations.
pub mod pg_error_codes {
    /// Unique constraint violation
    pub const UNIQUE_VIOLATION: &str = "23505";
    /// Foreign key constraint violation
    pub const FOREIGN_KEY_VIOLATION: &str = "23503";
    /// Not null constraint violation
    pub const NOT_NULL_VIOLATION: &str = "23502";
    /// Check constraint violation
    pub const CHECK_VIOLATION: &str = "23514";
}

/// Maps SQLx errors to domain RepositoryErrors with proper detection of constraint violations.
///
/// This function provides consistent error handling across all repositories:
/// - Detects unique constraint violations (23505) and returns `RepositoryError::Duplicate`
/// - Detects foreign key violations (23503) and returns `RepositoryError::Database` with context
/// - All other errors are wrapped in `RepositoryError::Database`
///
/// # Example
///
/// ```rust,ignore
/// use persistence::db_utils::map_db_error;
///
/// async fn create_user(&self, ...) -> Result<UserDto, RepositoryError> {
///     sqlx::query_as(...)
///         .fetch_one(&self.pool)
///         .await
///         .map_err(map_db_error)
/// }
/// ```
pub fn map_db_error(e: sqlx::Error) -> RepositoryError {
    if let Some(db_err) = e.as_database_error() {
        if let Some(code) = db_err.code() {
            match code.as_ref() {
                pg_error_codes::UNIQUE_VIOLATION => {
                    return RepositoryError::Duplicate(db_err.message().to_string());
                }
                pg_error_codes::FOREIGN_KEY_VIOLATION => {
                    return RepositoryError::Database(format!(
                        "Foreign key constraint violation: {}",
                        db_err.message()
                    ));
                }
                pg_error_codes::NOT_NULL_VIOLATION => {
                    return RepositoryError::Database(format!(
                        "Required field is missing: {}",
                        db_err.message()
                    ));
                }
                pg_error_codes::CHECK_VIOLATION => {
                    return RepositoryError::Database(format!(
                        "Validation constraint failed: {}",
                        db_err.message()
                    ));
                }
                _ => {}
            }
        }
    }
    RepositoryError::Database(e.to_string())
}

/// Maps SQLx errors with custom context for the entity type.
///
/// Similar to `map_db_error` but includes the entity name in error messages
/// for better debugging.
///
/// # Example
///
/// ```rust,ignore
/// use persistence::db_utils::map_db_error_with_context;
///
/// async fn create_site(&self, ...) -> Result<SiteDto, RepositoryError> {
///     sqlx::query_as(...)
///         .fetch_one(&self.pool)
///         .await
///         .map_err(|e| map_db_error_with_context(e, "Site"))
/// }
/// ```
pub fn map_db_error_with_context(e: sqlx::Error, entity: &str) -> RepositoryError {
    if let Some(db_err) = e.as_database_error() {
        if let Some(code) = db_err.code() {
            match code.as_ref() {
                pg_error_codes::UNIQUE_VIOLATION => {
                    return RepositoryError::Duplicate(format!(
                        "{} already exists: {}",
                        entity,
                        db_err.message()
                    ));
                }
                pg_error_codes::FOREIGN_KEY_VIOLATION => {
                    return RepositoryError::Database(format!(
                        "{}: Referenced entity not found - {}",
                        entity,
                        db_err.message()
                    ));
                }
                _ => {}
            }
        }
    }
    RepositoryError::Database(format!("{}: {}", entity, e))
}

/// Trait extension for converting SQLx Results to RepositoryError Results.
///
/// This provides a more ergonomic way to map errors in repository methods.
///
/// # Example
///
/// ```rust,ignore
/// use persistence::db_utils::SqlxResultExt;
///
/// async fn find_by_id(&self, id: Uuid) -> Result<Option<UserDto>, RepositoryError> {
///     sqlx::query_as(...)
///         .fetch_optional(&self.pool)
///         .await
///         .map_repo_err()
/// }
/// ```
pub trait SqlxResultExt<T> {
    fn map_repo_err(self) -> Result<T, RepositoryError>;
    fn map_repo_err_ctx(self, entity: &str) -> Result<T, RepositoryError>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn map_repo_err(self) -> Result<T, RepositoryError> {
        self.map_err(map_db_error)
    }

    fn map_repo_err_ctx(self, entity: &str) -> Result<T, RepositoryError> {
        self.map_err(|e| map_db_error_with_context(e, entity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_db_error_generic() {
        let err = sqlx::Error::RowNotFound;
        let result = map_db_error(err);
        assert!(matches!(result, RepositoryError::Database(_)));
    }
}
