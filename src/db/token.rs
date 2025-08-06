use sqlx::PgPool;

pub async fn add_token(pool: &PgPool, token: String, user_id: i32) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO
            session_token(token, user_id)
        VALUES
            ($1, $2)
        "#,
        token,
        user_id,
    ).execute(pool).await?;
    Ok(())
}

pub async fn exist_token(pool: &PgPool, token: String, user_id: i32) -> anyhow::Result<bool> {
    let rec = sqlx::query!(
        r#"
        SELECT count(*) FROM session_token WHERE token = $1 AND user_id = $2
        "#,
        token,
        user_id
    ).fetch_one(pool).await?;
    Ok(rec.count.unwrap() == 1)
}