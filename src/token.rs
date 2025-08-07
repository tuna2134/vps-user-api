use axum::{RequestPartsExt, extract::FromRequestParts, http::request::Parts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use base64::prelude::*;

use crate::{db::token::exist_token, error::APIError, state::AppState};

pub struct Token {
    pub user_id: i32,
    pub nonce: [u8; 32],
}

impl Token {
    pub fn new(user_id: i32) -> anyhow::Result<Self> {
        let mut nonce = [0; 32];
        getrandom::fill(&mut nonce)?;
        Ok(Self { user_id, nonce })
    }

    pub fn generate(&self) -> anyhow::Result<String> {
        let mut buffer = [0; 37];
        buffer[..4].copy_from_slice(&self.user_id.to_be_bytes());
        buffer[4] = b'.';
        buffer[5..].copy_from_slice(&self.nonce);
        Ok(BASE64_URL_SAFE_NO_PAD.encode(buffer))
    }

    pub fn parse(token: String) -> anyhow::Result<Self> {
        let buffer = BASE64_URL_SAFE_NO_PAD.decode(token.as_bytes())?;
        let mut user_id_bytes = [0u8; 4];
        user_id_bytes.copy_from_slice(&buffer[..4]);
        let user_id = i32::from_be_bytes(user_id_bytes);
        let mut nonce = [0u8; 32];
        nonce.copy_from_slice(&buffer[5..]);
        Ok(Self { user_id, nonce })
    }
}

impl FromRequestParts<AppState> for Token {
    type Rejection = APIError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| APIError::unauthorized("Missing authorization header"))?;

        let token = Token::parse(bearer.token().to_string())?;

        let nonce = BASE64_URL_SAFE_NO_PAD.encode(token.nonce);

        if !exist_token(&state.db_pool, nonce, token.user_id).await? {
            return Err(APIError::unauthorized("Invalid token"));
        }

        Ok(token)
    }
}
