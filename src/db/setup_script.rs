use sqlx::PgPool;

pub async fn db_create_setup_script(
    pool: &PgPool,
    title: String,
    description: Option<String>,
    script: String,
    author_id: i32,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO setup_script (title, description, script, author_id)
        VALUES ($1, $2, $3, $4)
        "#,
        title,
        description,
        script,
        author_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
