use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    db::setup_script::db_create_setup_script, error::APIResult, state::AppState, token::Token,
};

#[derive(Deserialize)]
pub struct CreateSetupScriptRequest {
    pub title: String,
    pub description: Option<String>,
    pub script: String,
}

pub async fn create_setup_script(
    State(state): State<AppState>,
    token: Token,
    Json(payload): Json<CreateSetupScriptRequest>,
) -> APIResult<()> {
    db_create_setup_script(
        &state.db_pool,
        payload.title,
        payload.description,
        payload.script,
        token.user_id,
    )
    .await?;
    Ok(())
}
