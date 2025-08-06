use std::sync::Arc;

use bb8_redis::{RedisConnectionManager, bb8};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
    pub redis_pool: Arc<bb8::Pool<RedisConnectionManager>>,
}

impl AppState {
    pub async fn connect(db_url: &str, redis_url: &str) -> anyhow::Result<Self> {
        let db_pool = PgPool::connect(db_url).await?;
        let redis_manager = RedisConnectionManager::new(redis_url)?;
        let redis_pool = bb8::Pool::builder().build(redis_manager).await?;

        Ok(AppState {
            db_pool: Arc::new(db_pool),
            redis_pool: Arc::new(redis_pool),
        })
    }
}
