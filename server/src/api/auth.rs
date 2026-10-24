use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::api::error::ApiError;

/// Extrator Axum que valida o cabeçalho Authorization: Bearer <TOKEN>.
pub struct RequireAuth;

impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| {
                ApiError::Unauthorized("Cabeçalho 'Authorization' ausente ou malformado".into())
            })?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized(
                "O formato de autorização deve ser 'Bearer <TOKEN>'".into(),
            ));
        }

        let token = &auth_header["Bearer ".len()..];
        let expected_token = std::env::var("BRIDGE_AUTH_TOKEN")
            .unwrap_or_else(|_| "secret-bridge-token".to_string());

        if token != expected_token {
            return Err(ApiError::Unauthorized("Token de autenticação inválido ou expirado".into()));
        }

        Ok(RequireAuth)
    }
}
