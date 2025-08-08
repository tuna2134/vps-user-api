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
        id,
        name,
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

pub async fn get_all_servers_from_user(
    pool: &PgPool,
    user_id: i32,
) -> anyhow::Result<Vec<(String, String, i32, String)>> {
    let servers = sqlx::query!(
        r#"
        SELECT
            id,
            name,
            plan,
            ip_address
        FROM
            server
        WHERE
            author_id = $1
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| (row.id, row.name, row.plan, row.ip_address))
    .collect();
    Ok(servers)
}

pub async fn db_get_server_by_id(pool: &PgPool, server_id: String, user_id: i32) -> anyhow::Result<Option<(String, String, i32, String)>> {
    let row = sqlx::query!(
        r#"
        SELECT
            id,
            name,
            plan,
            ip_address
        FROM
            server
        WHERE
            id = $1
        AND
            author_id = $2
        "#,
        server_id,
        user_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| (r.id, r.name, r.plan, r.ip_address)))
}