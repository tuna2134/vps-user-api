use std::env;

use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};

use crate::{
    db::server::{add_server, db_get_server_by_id, get_all_servers_from_user, get_server_ips},
    error::{APIError, APIResult},
    state::AppState,
    token::Token,
    utils::{
        api::domain::{
            create_domain, fetch_all_servers, fetch_server, CreateDomainRequest, CreateDomainRequestNetwork, CreateDomainRequestResources
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

#[derive(Deserialize, Serialize)]
pub struct CreateServerRequest {
    pub name: String,
    pub server_password: String,
    pub plan: i32,
}

pub async fn create_server(
    State(state): State<AppState>,
    token: Token,
    Json(payload): Json<CreateServerRequest>,
) -> APIResult<()> {
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
        fetch_all_servers(server_ids).await?
    };
    // 結合する、server_onlines.domainsにサーバのIDが含まれている場合はオンライン、それ以外はオフライン
    let server_online_set: std::collections::HashSet<String> =
        server_onlines.domains.into_iter().collect();
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
        .map(
            |(id, name, plan, ip_address, status)| GetServerResponse {
                id,
                name,
                plan,
                ip_address,
                status,
            },
        )
        .collect();
    Ok(Json(response))
}

pub async fn get_server_by_id(
    State(state): State<AppState>,
    token: Token,
    Path(server_id): Path<String>,
) -> APIResult<Json<GetServerResponse>> {
    let server = db_get_server_by_id(&state.db_pool, server_id, token.user_id).await?;
    if let Some((id, name, plan, ip_address)) = server {
        let server_model = fetch_server(id.clone()).await?;
        Ok(Json(GetServerResponse {
            id,
            name,
            plan,
            ip_address,
            status: server_model.status,
        }))
    } else {
        Err(APIError::not_found("Server not found"))
    }
}