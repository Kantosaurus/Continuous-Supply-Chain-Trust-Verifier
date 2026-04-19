//! Authentication module for JWT and API key authentication.

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sctv_core::TenantId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{ApiError, AppState};

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: Uuid,
    /// Tenant ID.
    pub tenant_id: Uuid,
    /// User email.
    pub email: String,
    /// User roles.
    pub roles: Vec<String>,
    /// Issued at timestamp.
    pub iat: i64,
    /// Expiration timestamp.
    pub exp: i64,
    /// Issuer.
    pub iss: String,
    /// Audience.
    pub aud: String,
}

impl Claims {
    /// Creates new claims for a user.
    pub fn new(
        user_id: Uuid,
        tenant_id: TenantId,
        email: String,
        roles: Vec<String>,
        issuer: &str,
        audience: &str,
        expires_in_hours: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            tenant_id: tenant_id.0,
            email,
            roles,
            iat: now,
            exp: now + (expires_in_hours * 3600),
            iss: issuer.to_string(),
            aud: audience.to_string(),
        }
    }

    /// Returns the tenant ID.
    pub fn tenant_id(&self) -> TenantId {
        TenantId(self.tenant_id)
    }

    /// Checks if the user has a specific role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Checks if the user is an admin.
    pub fn is_admin(&self) -> bool {
        self.has_role("admin")
    }
}

/// Authenticated user extracted from request.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub tenant_id: TenantId,
    pub email: String,
    pub roles: Vec<String>,
}

impl From<Claims> for AuthUser {
    fn from(claims: Claims) -> Self {
        Self {
            user_id: claims.sub,
            tenant_id: claims.tenant_id(),
            email: claims.email,
            roles: claims.roles,
        }
    }
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        // Check for Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        // Decode and validate JWT
        let claims = decode_token(token, &state.jwt_secret)?;

        Ok(AuthUser::from(claims))
    }
}

/// Optional authenticated user (doesn't fail if no auth present).
#[derive(Debug, Clone)]
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequestParts<Arc<AppState>> for MaybeAuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(user) => Ok(Self(Some(user))),
            Err(_) => Ok(Self(None)),
        }
    }
}

/// Encodes a JWT token.
pub fn encode_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let key = EncodingKey::from_secret(secret.as_bytes());
    encode(&Header::default(), claims, &key)
}

/// Decodes and validates a JWT token.
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, ApiError> {
    let key = DecodingKey::from_secret(secret.as_bytes());
    let mut validation = Validation::default();
    validation.set_audience(&["sctv"]);
    validation.set_issuer(&["sctv-api"]);

    let token_data = decode::<Claims>(token, &key, &validation)
        .map_err(|_| ApiError::Unauthorized)?;

    Ok(token_data.claims)
}

/// API key authentication (alternative to JWT).
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub key_id: Uuid,
    pub tenant_id: TenantId,
    pub scopes: Vec<String>,
}

impl FromRequestParts<Arc<AppState>> for ApiKeyAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        _parts: &mut Parts,
        _state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // API key authentication is not yet wired up to the database. Reject all
        // requests until the api_keys table and lookup path land (see plan item
        // Tier 4 #22). A prior placeholder accepted any 32+ char string, which
        // was a security bypass — do NOT restore that behavior.
        Err(ApiError::Unauthorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_token() {
        let claims = Claims::new(
            Uuid::new_v4(),
            TenantId::new(),
            "test@example.com".to_string(),
            vec!["user".to_string()],
            "sctv-api",
            "sctv",
            24,
        );

        let secret = "test-secret";
        let token = encode_token(&claims, secret).unwrap();
        let decoded = decode_token(&token, secret).unwrap();

        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.email, claims.email);
    }

    #[test]
    fn test_claims_roles() {
        let claims = Claims::new(
            Uuid::new_v4(),
            TenantId::new(),
            "admin@example.com".to_string(),
            vec!["user".to_string(), "admin".to_string()],
            "sctv-api",
            "sctv",
            24,
        );

        assert!(claims.has_role("admin"));
        assert!(claims.has_role("user"));
        assert!(!claims.has_role("superadmin"));
        assert!(claims.is_admin());
    }
}
