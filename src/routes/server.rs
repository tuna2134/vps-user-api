use std::env;

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

use crate::{
    db::server::{add_server, get_all_servers_from_user, get_server_ips},
    error::{APIError, APIResult},
    state::AppState,
    token::Token,
    utils::{
        api::domain::{
            CreateDomainRequest, CreateDomainRequestNetwork, CreateDomainRequestResources,
            create_domain,
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
pub struct GetAllServersResponse {
    pub id: String,
    pub name: String,
    pub plan: i32,
    pub ip_address: String,
}

pub async fn get_all_servers(
    State(state): State<AppState>,
    token: Token,
) -> APIResult<Json<Vec<GetAllServersResponse>>> {
    let servers = get_all_servers_from_user(&state.db_pool, token.user_id).await?;
    let response = servers
        .into_iter()
        .map(|(id, name, plan, ip_address)| GetAllServersResponse {
            id,
            name,
            plan,
            ip_address,
        })
        .collect();
    Ok(Json(response))
}
