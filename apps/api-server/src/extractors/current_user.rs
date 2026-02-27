//! Current User Extractor
//!
//! Extrator Axum para obter o usuário autenticado da requisição.
//! O usuário é inserido nas extensões pelo middleware de autenticação.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;

/// Representa o usuário autenticado extraído do JWT.
///
/// Este struct é inserido nas extensões da requisição pelo
/// middleware `mw_authenticate` após validação do token JWT.
///
/// # Uso em Handlers
///
/// ```rust,ignore
/// use crate::extractors::current_user::CurrentUser;
///
/// pub async fn my_handler(
///     CurrentUser(user): CurrentUser,
/// ) -> impl IntoResponse {
///     println!("User ID: {}", user.id);
///     println!("Username: {}", user.username);
///     // ...
/// }
/// ```
///
/// # Nota
///
/// Se o usuário não estiver autenticado (extensão não encontrada),
/// retorna `StatusCode::UNAUTHORIZED` (401), indicando que a rota
/// requer autenticação.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    /// UUID do usuário
    pub id: Uuid,
    /// Nome de usuário
    pub username: String,
}

impl CurrentUser {
    /// Cria uma nova instância de CurrentUser
    pub fn new(id: Uuid, username: String) -> Self {
        Self { id, username }
    }
}

/// Implementação do extrator para uso como parâmetro de handler.
///
/// Permite usar `CurrentUser` como parâmetro em handlers Axum:
///
/// ```rust,ignore
/// pub async fn handler(CurrentUser(user): CurrentUser) { ... }
/// // ou
/// pub async fn handler(user: CurrentUser) { ... }
/// ```
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CurrentUser>()
            .cloned()
            .ok_or_else(|| {
                let method = parts.method.as_str();
                let uri = parts.uri.path();
                tracing::warn!(
                    method = method,
                    uri = uri,
                    "CurrentUser não encontrado nas extensões — rota requer autenticação."
                );
                StatusCode::UNAUTHORIZED
            })
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_user_creation() {
        let user_id = Uuid::new_v4();
        let username = "testuser".to_string();

        let user = CurrentUser::new(user_id, username.clone());

        assert_eq!(user.id, user_id);
        assert_eq!(user.username, username);
    }
}
