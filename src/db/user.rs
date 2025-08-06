use sqlx::PgPool;

pub async fn add_user(
    pool: &PgPool,
    username: String,
    email: String,
    password_hash: String,
) -> anyhow::Result<i32> {
    let r = sqlx::query!(
        r#"
        INSERT INTO users (username, email, password_hash)
        VALUES ($1, $2, $3)
        RETURNING id
        "#,
        username,
        email,
        password_hash,
    )
    .fetch_one(pool)
    .await?;
    Ok(r.id)
}
