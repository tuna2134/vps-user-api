use sqlx::PgPool;

pub async fn add_server(
    pool: &PgPool,
    name: String,
    ip_address: String,
    server_type: String,
    author_id: i32,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO
            server (name, ip_address, type, author_id)
        VALUES
            ($1, $2, $3, $4)
        "#,
        name,
        ip_address,
        server_type,
        author_id
    ).execute(pool)
    .await?;
    Ok(())
}