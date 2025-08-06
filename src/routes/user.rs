use crate::{
    db::{token::add_token, user::{add_user, get_userid_by_name_and_password}},
    error::{APIError, APIResult},
    state::AppState, token::Token,
};
use axum::{Json, extract::State};
use base64::prelude::*;
use bb8_redis::redis::AsyncCommands;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

#[derive(Deserialize, Serialize)]
pub struct CreateUserRequestModel {
    pub username: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateUserResponseModel {
    pub token: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequestModel>,
) -> APIResult<Json<CreateUserResponseModel>> {
    let code: String = {
        let mut rng = rand::rng();
        (0..6)
            .map(|_| rng.random_range(0..10).to_string())
            .collect()
    };
    // TODO: Send email with the code
    tracing::info!("Generated code for user {}: {}", payload.username, code);
    let token: String = {
        let mut buf = [0u8; 32];
        getrandom::fill(&mut buf)?;
        BASE64_URL_SAFE_NO_PAD.encode(&buf)
    };
    {
        let mut conn = state.redis_pool.get().await?;
        let key = format!("create_user:{}", token);
        let value = serde_json::to_string(&payload)?;
        let _: () = conn.set_ex(key, value, 3600).await?;
    }
    Ok(Json(CreateUserResponseModel { token }))
}

#[derive(Deserialize, Serialize)]
pub struct RegisterUserRequestModel {
    pub token: String,
    pub code: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct RegisterUserResponseModel {
    pub token: String,
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequestModel>,
) -> APIResult<Json<RegisterUserResponseModel>> {
    let mut conn = state.redis_pool.get().await?;
    let key = format!("create_user:{}", payload.token);
    let userdata: Option<CreateUserRequestModel> = {
        let value: Option<String> = conn.get(key).await?;
        if let Some(value) = &value {
            Some(serde_json::from_str(value)?)
        } else {
            None
        }
    };
    if let Some(userdata) = &userdata {
        let password_hash = {
            let mut hasher = Sha256::new();
            hasher.update(payload.password.into_bytes());
            let hash = hasher.finalize();
            BASE64_URL_SAFE_NO_PAD.encode(hash)
        };
        let user_id = add_user(
            &state.db_pool,
            userdata.username.clone(),
            userdata.email.clone(),
            password_hash,
        )
        .await?;
        let token = Token::new(user_id)?;
        let token_str = token.generate()?;
        add_token(&state.db_pool, token_str.clone(), user_id).await?;
        Ok(Json(RegisterUserResponseModel { token: token_str }))
    } else {
        Err(APIError::not_found("User data not found").into())
    }
}

#[derive(Deserialize)]
pub struct IssueUserTokenRequestModel {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct IssueUserTokenResponseModel {
    pub token: String,
}

pub async fn issue_user_token(
    State(state): State<AppState>,
    Json(payload): Json<IssueUserTokenRequestModel>,
) -> APIResult<Json<IssueUserTokenResponseModel>> {
    let password_hash = {
        let mut hasher = Sha256::new();
        hasher.update(payload.password.into_bytes());
        let hash = hasher.finalize();
        BASE64_URL_SAFE_NO_PAD.encode(hash)
    };
    let user_id = get_userid_by_name_and_password(
        &state.db_pool,
        payload.email,
        password_hash,
    )
    .await?
    .ok_or_else(|| APIError::unauthorized("Invalid email or password"))?;
    let token = Token::new(user_id)?;
    let token_str = token.generate()?;
    add_token(&state.db_pool, token_str.clone(), user_id).await?;
    Ok(Json(IssueUserTokenResponseModel { token: token_str }))
}