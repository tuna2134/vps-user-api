use crate::{error::APIResult, state::AppState};
use axum::Json;
use base64::prelude::*;
use bb8_redis::redis::AsyncCommands;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
    state: AppState,
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
