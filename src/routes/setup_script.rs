use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    db::setup_script::{db_create_setup_script, db_get_all_setup_scripts},
    error::APIResult,
    state::AppState,
    token::Token,
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

#[derive(Serialize)]
pub struct GetSetupScriptResponse {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub script: String,
}

pub async fn get_all_setup_scripts(
    State(state): State<AppState>,
) -> APIResult<Json<Vec<GetSetupScriptResponse>>> {
    let scripts = db_get_all_setup_scripts(&state.db_pool).await?;
    Ok(Json(
        scripts
            .iter()
            .map(|(id, title, description, script)| GetSetupScriptResponse {
                id: *id,
                title: title.to_string(),
                description: description.clone(),
                script: script.to_string(),
            })
            .collect(),
    ))
}
