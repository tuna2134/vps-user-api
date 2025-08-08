use std::env;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CreateDomainRequest {
    pub password: String,
    pub network: CreateDomainRequestNetwork,
    pub resources: CreateDomainRequestResources,
}

#[derive(Serialize)]
pub struct CreateDomainRequestNetwork {
    pub address: String,
    pub gateway: String,
    pub interface: String,
}

#[derive(Serialize)]
pub struct CreateDomainRequestResources {
    pub cpu: i32,
    pub memory: i32,
    pub disk: String,
}

#[derive(Deserialize)]
pub struct CreateDomainResponse {
    pub id: String,
}

pub async fn create_domain(payload: CreateDomainRequest) -> anyhow::Result<String> {
    let response = reqwest::Client::new()
        .post(format!("{}/domains", env::var("VM_CONTROLLER_ENDPOINT")?))
        .json(&payload)
        .send()
        .await?;
    if !response.status().is_success() {
        return anyhow::bail!("Failed to create domain: {}", response.status());
    }
    let response_body: CreateDomainResponse = response.json().await?;
    Ok(response_body.id)
}

#[derive(Deserialize)]
pub struct AddServerResponse {
    pub domains: Vec<String>,
}

pub async fn fetch_all_servers(server_ids: Vec<String>) -> anyhow::Result<AddServerResponse> {
    let response = reqwest::Client::new()
        .get(format!("{}/domains", env::var("VM_CONTROLLER_ENDPOINT")?))
        .query(&[("running", "true"), ("ids", &server_ids.join(","))])
        .send()
        .await?;
    if !response.status().is_success() {
        return anyhow::bail!("Failed to get all servers: {}", response.status());
    }
    let response_body: AddServerResponse = response.json().await?;
    Ok(response_body)
}

#[derive(Deserialize)]
pub struct ServerModel {
    pub status: String,
}

pub async fn fetch_server(server_id: String) -> anyhow::Result<ServerModel> {
    let response = reqwest::Client::new()
        .get(format!(
            "{}/domains/{}",
            env::var("VM_CONTROLLER_ENDPOINT")?,
            server_id
        ))
        .send()
        .await?;
    if !response.status().is_success() {
        return anyhow::bail!("Failed to fetch server: {}", response.status());
    }
    let response_body: ServerModel = response.json().await?;
    Ok(response_body)
}
