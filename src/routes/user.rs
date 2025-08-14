use crate::{
    db::{
        token::add_token,
        user::{add_user, get_userdata_by_id, get_userid_by_email_and_password},
    },
    error::{APIError, APIResult},
    state::AppState,
    token::Token,
    utils::mail::send_passcode,
};
use axum::{Json, extract::State};
use base64::prelude::*;
use bb8_redis::redis::AsyncCommands;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize)]
pub struct CreateUserRequestModel {
    pub username: String,
    pub email: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateUserResponseModel {
    pub token: String,
}

// 仮ユーザーを作成します。
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
    send_passcode(code.clone(), payload.email.clone()).await?;
    tracing::info!("Generated code for user {}: {}", payload.username, code);
    let token: String = {
        let mut buf = [0u8; 32];
        getrandom::fill(&mut buf)?;
        BASE64_URL_SAFE_NO_PAD.encode(buf)
    };
    {
        let mut conn = state.redis_pool.get().await?;
        let key = format!("create_user:{token}");
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

// ユーザーの本登録です。
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
        let nonce = token.get_nonce_as_string();
        add_token(&state.db_pool, nonce, user_id).await?;
        Ok(Json(RegisterUserResponseModel {
            token: token.generate()?,
        }))
    } else {
        Err(APIError::not_found("User data not found"))
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

// ユーザーのトークンを発行します。(ログインで主に利用します。)
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
    let user_id = get_userid_by_email_and_password(&state.db_pool, payload.email, password_hash)
        .await?
        .ok_or_else(|| APIError::unauthorized("Invalid email or password"))?;
    let token = Token::new(user_id)?;
    let nonce = token.get_nonce_as_string();
    add_token(&state.db_pool, nonce, user_id).await?;
    Ok(Json(IssueUserTokenResponseModel {
        token: token.generate()?,
    }))
}

#[derive(Serialize)]
pub struct GetUserDataResponseModel {
    pub username: String,
    pub email: String,
    pub avatar_url: String,
    pub id: i32,
}

// ユーザーのデータを取得します。
pub async fn get_user(
    State(state): State<AppState>,
    token: Token,
) -> APIResult<Json<GetUserDataResponseModel>> {
    if let Some((username, email)) = get_userdata_by_id(&state.db_pool, token.user_id).await? {
        let avatar_url = {
            let mut hasher = Sha256::new();
            hasher.update(email.as_bytes());
            let hash = hasher.finalize();
            format!("https://gravatar.com/avatar/{hash:x}")
        };
        Ok(Json(GetUserDataResponseModel {
            username,
            email,
            avatar_url,
            id: token.user_id,
        }))
    } else {
        Err(APIError::not_found("User not found"))
    }
}
