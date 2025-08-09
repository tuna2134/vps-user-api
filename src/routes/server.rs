use std::env;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{
        server::{
            add_server, db_delete_server_by_id, db_get_server_by_id, get_all_servers_from_user,
            get_server_ips,
        },
        setup_script::get_script_by_id,
    },
    error::{APIError, APIResult},
    state::AppState,
    token::Token,
    utils::{
        api::domain::{
            self, CreateDomainRequest, CreateDomainRequestNetwork, CreateDomainRequestResources,
            create_domain, fetch_all_servers, fetch_server,
        },
        ip_calc::cidr_to_list,
    },
};

#[derive(Serialize, Deserialize)]
pub struct ServerPlanResource {
    pub cpu: i32,
    pub memory: i32,
    pub disk: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ServerPlan {
    pub id: i32,
    pub name: String,
    pub resources: ServerPlanResource,
}

#[derive(Deserialize, Serialize)]
pub struct ServerPlansResponse {
    pub plans: Vec<ServerPlan>,
}

pub async fn get_server_plans() -> APIResult<Json<ServerPlansResponse>> {
    let data: ServerPlansResponse = serde_json::from_str(include_str!("../../data.json"))?;
    Ok(Json(data))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateServerRequest {
    pub name: String,
    pub server_password: String,
    pub plan: i32,
    pub script_id: Option<i32>,
}

pub async fn create_server(
    State(state): State<AppState>,
    token: Token,
    Json(payload): Json<CreateServerRequest>,
) -> APIResult<()> {
    tracing::debug!("Creating server with payload: {:?}", payload);
    let mut used_ips = get_server_ips(&state.db_pool).await?;
    let (ips, prefix) = cidr_to_list(&env::var("NETWORK_CIDR")?)?;
    used_ips.push(format!("{}/{}", env::var("NETWORK_GATEWAY")?, prefix));
    let available_ips: Vec<String> = ips
        .iter()
        .filter(|ip| !used_ips.contains(ip))
        .map(|ip| ip.to_string())
        .collect();
    let ip_address = available_ips
        .first()
        .ok_or_else(|| APIError::bad_request("No available IP addresses"))?;
    let plan: ServerPlan = {
        let data: ServerPlansResponse =
            serde_json::from_str(include_str!("../../data.json")).unwrap();
        data.plans
            .into_iter()
            .find(|p| p.id == payload.plan)
            .unwrap()
    };
    let script: Option<String> = if let Some(script_id) = payload.script_id {
        get_script_by_id(&state.db_pool, script_id).await?
    } else {
        None
    };
    tracing::debug!("{:?}", script);
    let server_id = create_domain(CreateDomainRequest {
        password: payload.server_password.clone(),
        network: CreateDomainRequestNetwork {
            address: ip_address.clone(),
            gateway: env::var("NETWORK_GATEWAY")?,
            interface: env::var("NETWORK_INTERFACE")?,
        },
        resources: CreateDomainRequestResources {
            cpu: plan.resources.cpu,
            memory: plan.resources.memory / 1024,
            disk: format!("{}G", plan.resources.disk),
        },
        script,
    })
    .await?;
    add_server(
        &state.db_pool,
        server_id,
        payload.name,
        ip_address.to_string(),
        payload.plan,
        token.user_id,
    )
    .await
    .map_err(|e| APIError::internal_server_error(&e.to_string()))?;
    Ok(())
}

#[derive(Serialize)]
pub struct GetServerResponse {
    pub id: String,
    pub name: String,
    pub plan: i32,
    pub ip_address: String,
    pub status: String,
}

// ユーザーが所有するサーバーの一覧を取得します。
pub async fn get_all_servers(
    State(state): State<AppState>,
    token: Token,
) -> APIResult<Json<Vec<GetServerResponse>>> {
    let servers = get_all_servers_from_user(&state.db_pool, token.user_id).await?;
    let server_onlines = {
        let server_ids: Vec<String> = servers.iter().map(|(id, _, _, _)| id.clone()).collect();
        fetch_all_servers(server_ids)
            .await?
            .domains
            .unwrap_or_default()
    };
    // 結合する、server_onlines.domainsにサーバのIDが含まれている場合はオンライン、それ以外はオフライン
    let server_online_set: std::collections::HashSet<String> = server_onlines.into_iter().collect();
    let servers: Vec<_> = servers
        .into_iter()
        .map(|(id, name, plan, ip_address)| {
            let status = if server_online_set.contains(&id) {
                "online".to_string()
            } else {
                "offline".to_string()
            };
            (id, name, plan, ip_address, status)
        })
        .collect();
    let response = servers
        .into_iter()
        .map(|(id, name, plan, ip_address, status)| GetServerResponse {
            id,
            name,
            plan,
            ip_address,
            status,
        })
        .collect();
    Ok(Json(response))
}

pub async fn get_server_by_id(
    State(state): State<AppState>,
    token: Token,
    Path((server_id,)): Path<(String,)>,
) -> APIResult<Json<GetServerResponse>> {
    let server = db_get_server_by_id(&state.db_pool, server_id, token.user_id).await?;
    if let Some((id, name, plan, ip_address)) = server {
        let server_model = fetch_server(id.clone()).await?;
        Ok(Json(GetServerResponse {
            id,
            name,
            plan,
            ip_address,
            status: if server_model.status == "running" {
                "online".to_string()
            } else {
                "offline".to_string()
            },
        }))
    } else {
        Err(APIError::not_found("Server not found"))
    }
}

pub async fn delete_server(
    State(state): State<AppState>,
    token: Token,
    Path((server_id,)): Path<(String,)>,
) -> APIResult<()> {
    domain::delete_server(server_id.clone()).await?;
    db_delete_server_by_id(&state.db_pool, server_id, token.user_id).await?;
    Ok(())
}

pub async fn shutdown_server(
    State(state): State<AppState>,
    token: Token,
    Path((server_id,)): Path<(String,)>,
) -> APIResult<()> {
    if db_get_server_by_id(&state.db_pool, server_id.clone(), token.user_id)
        .await?
        .is_none()
    {
        return Err(APIError::not_found("Server not found"));
    }

    domain::shutdown_server(server_id).await?;
    Ok(())
}

pub async fn power_on_server(
    State(state): State<AppState>,
    token: Token,
    Path((server_id,)): Path<(String,)>,
) -> APIResult<()> {
    if db_get_server_by_id(&state.db_pool, server_id.clone(), token.user_id)
        .await?
        .is_none()
    {
        return Err(APIError::not_found("Server not found"));
    }

    domain::power_on_server(server_id).await?;
    Ok(())
}

pub async fn restart_server(
    State(state): State<AppState>,
    token: Token,
    Path((server_id,)): Path<(String,)>,
) -> APIResult<()> {
    if db_get_server_by_id(&state.db_pool, server_id.clone(), token.user_id)
        .await?
        .is_none()
    {
        return Err(APIError::not_found("Server not found"));
    }

    domain::restart_server(server_id).await?;
    Ok(())
}
