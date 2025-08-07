use sqlx::PgPool;

pub async fn add_server(
    pool: &PgPool,
    id: String,
    name: String,
    ip_address: String,
    plan: i32,
    author_id: i32,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO
            server (id, name, ip_address, plan, author_id)
        VALUES
            ($1, $2, $3, $4, $5)
        "#,
        name,
        id,
        ip_address,
        plan,
        author_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_server_ips(pool: &PgPool) -> anyhow::Result<Vec<String>> {
    let ips = sqlx::query!("SELECT ip_address FROM server")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.ip_address)
        .collect();
    Ok(ips)
}
