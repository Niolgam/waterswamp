pub mod contracts;
pub mod handlers;

use axum::{
    routing::{get, put},
    Router,
};

use crate::infra::state::AppState;

pub use contracts::{
    ChangePasswordRequest, ChangePasswordResponse, ProfileResponse, UpdateProfileRequest,
};

/// use crate::api::users;
/// use crate::middleware::mw_authenticate;
///
/// let router = users::router()
///     .layer(middleware::from_fn_with_state(state, mw_authenticate));
/// ```
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users/profile", get(handlers::get_profile))
        .route("/users/profile", put(handlers::update_profile))
        .route("/users/password", put(handlers::change_password))
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let _router: Router<AppState> = router();
    }
}
