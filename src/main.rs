use std::env;

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;

use crate::{
    routes::{
        server::{create_server, get_server_plans},
        user::register_user,
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

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/users", post(routes::user::create_user))
        .route("/users/register", post(register_user))
        .route("/users/login", post(routes::user::issue_user_token))
        .route("/servers/plans", get(get_server_plans))
        .route("/servers", post(create_server))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
