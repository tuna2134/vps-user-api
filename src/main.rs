use std::env;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    routes::{
        server::{create_server, get_all_servers, get_server_plans},
        setup_script::{create_setup_script, get_all_setup_scripts},
        user::{get_user, register_user},
    },
    state::AppState,
};

mod db;
mod error;
mod routes;
mod state;
mod token;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let state = AppState::connect(&env::var("DATABASE_URL")?, &env::var("REDIS_URL")?).await?;

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_origin(Any);

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/users", post(routes::user::create_user))
        .route("/users/@me", get(get_user))
        .route("/users/register", post(register_user))
        .route("/users/login", post(routes::user::issue_user_token))
        .route("/servers/plans", get(get_server_plans))
        .route("/servers", post(create_server))
        .route("/servers/{id}", get(routes::server::get_server_by_id))
        .route("/servers/{id}", delete(routes::server::delete_server))
        .route(
            "/servers/{id}/shutdown",
            post(routes::server::shutdown_server),
        )
        .route(
            "/servers/{id}/power_on",
            post(routes::server::power_on_server),
        )
        .route(
            "/servers/{id}/restart",
            post(routes::server::restart_server),
        )
        .route("/users/@me/servers", get(get_all_servers))
        .route("/setup-scripts", post(create_setup_script))
        .route("/setup-scripts", get(get_all_setup_scripts))
        .route(
            "/setup-scripts/{id}",
            get(routes::setup_script::get_script_by_id),
        )
        .route(
            "/setup-scripts/{id}",
            put(routes::setup_script::put_script_script),
        )
        .route(
            "/setup-scripts/{id}",
            delete(routes::setup_script::delete_script),
        )
        .layer(cors)
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
