use std::env;

use axum::{
    Router,
    routing::{get, post},
};
use tokio::net::TcpListener;

use crate::state::AppState;

mod error;
mod routes;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    let state = AppState::connect(&env::var("DATABASE_URL")?, &env::var("REDIS_URL")?).await?;

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/user", post(routes::user::create_user))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, router).await?;
    Ok(())
}
