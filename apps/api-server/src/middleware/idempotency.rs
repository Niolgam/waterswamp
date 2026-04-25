use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use std::future::Future;
use uuid::Uuid;

/// Extrai e valida o header `Idempotency-Key` (UUID v4) — DRS 4.4.
///
/// Retorna 400 se o header estiver ausente ou não for um UUID válido.
pub struct IdempotencyKey(pub Uuid);

impl<S> FromRequestParts<S> for IdempotencyKey
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = parts
            .headers
            .get("Idempotency-Key")
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "Header 'Idempotency-Key' obrigatório (UUID v4)".to_string(),
                )
            })
            .and_then(|v| {
                v.to_str().map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        "Header 'Idempotency-Key' contém caracteres inválidos".to_string(),
                    )
                })
            })
            .and_then(|s| {
                Uuid::parse_str(s).map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("'{}' não é um UUID v4 válido", s),
                    )
                })
            })
            .map(IdempotencyKey);

        std::future::ready(result)
    }
}
