use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::APIResult;

#[derive(Serialize, Deserialize)]
pub struct ServerPlanResource {
    pub cpu: i32,
    pub memory: i32,
    pub disk: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ServerPlan {
    pub id: String,
    pub name: String,
    pub resources: ServerPlanResource,
}

#[derive(Deserialize, Serialize)]
pub struct ServerPlansResponse {
    pub plans: Vec<ServerPlan>,
}

pub async fn get_server_plans() -> APIResult<Json<ServerPlansResponse>> {
    let data: ServerPlansResponse = serde_json::from_str("data.json")?;
    Ok(Json(data))
}