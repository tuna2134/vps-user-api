use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::setup_script::{db_create_setup_script, db_get_all_setup_scripts, get_scriptdata_by_id, set_setup_script},
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
    pub author_id: i32,
}

pub async fn get_all_setup_scripts(
    State(state): State<AppState>,
    _token: Token,
) -> APIResult<Json<Vec<GetSetupScriptResponse>>> {
    let scripts = db_get_all_setup_scripts(&state.db_pool).await?;
    Ok(Json(
        scripts
            .iter()
            .map(
                |(id, title, description, script, author_id)| GetSetupScriptResponse {
                    id: *id,
                    title: title.to_string(),
                    description: description.clone(),
                    script: script.to_string(),
                    author_id: *author_id,
                },
            )
            .collect(),
    ))
}

pub async fn get_script_by_id(
    State(state): State<AppState>,
    _token: Token,
    Path((script_id,)): Path<(i32,)>,
) -> APIResult<Json<GetSetupScriptResponse>> {
    if let Some((title, description, script, author_id)) =
        get_scriptdata_by_id(&state.db_pool, script_id).await?
    {
        Ok(Json(GetSetupScriptResponse {
            id: script_id,
            title,
            description,
            script,
            author_id,
        }))
    } else {
        Err(anyhow::anyhow!("Script not found").into())
    }
}

pub async fn put_script_script(
    State(state): State<AppState>,
    token: Token,
    Path((script_id,)): Path<(i32,)>,
    Json(payload): Json<CreateSetupScriptRequest>,
) -> APIResult<()> {
    set_setup_script(
        &state.db_pool,
        script_id,
        token.user_id,
        payload.title,
        payload.description,
        payload.script,
    )
    .await?;
    Ok(())
}