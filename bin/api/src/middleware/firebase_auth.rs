// middleware/firebase_auth.rs
use axum::{Json, extract::Request, http::StatusCode, middleware::Next, response::Response};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error};

use crate::state::AppState;

const FIREBASE_JWKS_URL: &str =
    "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirebaseClaims {
    pub sub: String, // Firebase UID
    pub email: Option<String>,
    pub aud: String, // Firebase project ID
    pub iat: usize,
    pub exp: usize,
}

pub async fn firebase_auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<String>)> {
    // 1. Extract Bearer token
    let token = extract_bearer_token(&request)?;

    // 2. Get the key ID from the JWT header
    let header = decode_header(&token).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json("Invalid token header".into()),
        )
    })?;

    let kid = header.kid.ok_or((
        StatusCode::UNAUTHORIZED,
        Json("Missing kid in token header".into()),
    ))?;

    // 3. Get keys from cache — refresh if stale
    let keys = get_keys(&state).await?;

    // 4. Find matching public key — if missing, force refresh and retry once
    //    (handles the case where Firebase rotated keys since last fetch)
    let public_key = match keys.get(&kid) {
        Some(k) => k.clone(),
        None => {
            debug!("kid not found in cache, forcing key refresh");
            let fresh_keys = refresh_keys(&state).await?;
            fresh_keys.get(&kid).cloned().ok_or((
                StatusCode::UNAUTHORIZED,
                Json("Unknown token signing key".into()),
            ))?
        }
    };

    // 5. Validate JWT signature, expiry, audience, issuer
    let claims = validate_token(&token, &public_key)?;

    // 6. Attach claims to request extensions — handlers can opt-in via Extension<FirebaseClaims>
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn extract_bearer_token(request: &Request) -> Result<String, (StatusCode, Json<String>)> {
    request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t.to_string())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            Json("Missing or invalid Authorization header".into()),
        ))
}

async fn get_keys(state: &AppState) -> Result<HashMap<String, String>, (StatusCode, Json<String>)> {
    let cache = state.firebase_keys.read().await;
    if cache.is_stale() {
        // Drop read lock before acquiring write lock
        drop(cache);
        return refresh_keys(state).await;
    }
    debug!("Keys in cache: {:?}", cache.keys.keys().collect::<Vec<_>>());

    Ok(cache.keys.clone())
}

async fn refresh_keys(
    state: &AppState,
) -> Result<HashMap<String, String>, (StatusCode, Json<String>)> {
    let fresh_keys = fetch_firebase_keys().await.map_err(|e| {
        error!("Failed to refresh Firebase keys: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Failed to refresh Firebase keys".into()),
        )
    })?;

    let mut cache = state.firebase_keys.write().await;
    cache.keys = fresh_keys.clone();
    cache.fetched_at = std::time::Instant::now();

    Ok(fresh_keys)
}

pub async fn fetch_firebase_keys() -> anyhow::Result<HashMap<String, String>> {
    let keys: HashMap<String, String> = reqwest::get(FIREBASE_JWKS_URL).await?.json().await?;
    Ok(keys)
}

fn validate_token(
    token: &str,
    public_key_pem: &str,
) -> Result<FirebaseClaims, (StatusCode, Json<String>)> {
    let project_id = std::env::var("RUSTIC_AI_FIREBASE_PROJECT_ID").map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("FIREBASE_PROJECT_ID env var not set".into()),
        )
    })?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[&project_id]);
    validation.set_issuer(&[format!("https://securetoken.google.com/{}", project_id)]);

    let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes()).map_err(|e| {
        error!("Failed to parse Firebase public key: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json("Failed to parse Firebase public key".into()),
        )
    })?;

    decode::<FirebaseClaims>(token, &decoding_key, &validation)
        .map(|data| data.claims)
        .map_err(|e| {
            debug!("Token validation failed: {}", e);
            (
                StatusCode::UNAUTHORIZED,
                Json(format!("Invalid token: {}", e)),
            )
        })
}
