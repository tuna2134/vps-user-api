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

pub async fn db_get_all_setup_scripts(
    pool: &PgPool,
) -> anyhow::Result<Vec<(i32, String, Option<String>, String)>> {
    let scripts = sqlx::query!(
        r#"
        SELECT id, title, description, script FROM setup_script
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| (row.id, row.title, row.description, row.script))
    .collect();
    Ok(scripts)
}
