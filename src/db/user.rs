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

pub async fn get_userid_by_name_and_password(
    pool: &PgPool,
    username: String,
    password_hash: String,
) -> anyhow::Result<Option<i32>> {
    let rec = sqlx::query!(
        r#"
        SELECT
            id
        FROM
            users
        WHERE
            username = $1
        AND
            password_hash = $2
        "#,
        username,
        password_hash
    )
    .fetch_optional(pool)
    .await?;

    Ok(rec.map(|r| r.id))
}

pub async fn get_userdata_by_id(
    pool: &PgPool,
    user_id: i32,
) -> anyhow::Result<Option<(String, String)>> {
    let rec = sqlx::query!(
        r#"
        SELECT
            username,
            email
        FROM
            users
        WHERE
            id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(rec.map(|r| (r.username, r.email)))
}