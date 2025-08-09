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
) -> anyhow::Result<Vec<(i32, String, Option<String>, String, i32)>> {
    let scripts = sqlx::query!(
        r#"
        SELECT
            id, title, description, script, author_id
        FROM
            setup_script
        "#,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| {
        (
            row.id,
            row.title,
            row.description,
            row.script,
            row.author_id,
        )
    })
    .collect();
    Ok(scripts)
}

pub async fn get_script_by_id(pool: &PgPool, script_id: i32) -> anyhow::Result<Option<String>> {
    let script = sqlx::query!(
        r#"
        SELECT script FROM setup_script WHERE id = $1
        "#,
        script_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(script.map(|row| row.script))
}

pub async fn get_scriptdata_by_id(
    pool: &PgPool,
    script_id: i32,
) -> anyhow::Result<Option<(String, Option<String>, String, i32)>> {
    let script = sqlx::query!(
        r#"
        SELECT title, description, script, author_id FROM setup_script WHERE id = $1
        "#,
        script_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(script.map(|row| (row.title, row.description, row.script, row.author_id)))
}
