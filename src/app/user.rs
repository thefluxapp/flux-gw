use anyhow::Error;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt as _,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, TokenData, Validation};
use serde::Deserialize;
use uuid::Uuid;

use super::{error::AppError, state::AppState};

#[async_trait]
impl<S> FromRequestParts<S> for AppUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AppState { public_key, .. } = AppState::from_ref(state);

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await?;

        let user = extract_user(bearer, &public_key).await?;

        Ok(user)
    }
}

async fn extract_user(bearer: Bearer, public_key_file: &Vec<u8>) -> Result<AppUser, Error> {
    let TokenData { claims, .. } = decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_rsa_pem(public_key_file)?,
        &Validation::new(Algorithm::RS256),
    )?;

    Ok(AppUser { id: claims.sub })
}

#[derive(Deserialize)]
pub struct AppUser {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
}