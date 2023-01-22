use serde::Serializer;
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    types::chrono,
};
use std::env;
use uuid::Uuid;

pub mod shop;
pub mod users;

pub async fn connect(database: &str) -> Result<PgPool, sqlx::Error> {
    let mut uri = env::var("DATABASE_URL").expect("DATABASE_URL not present");
    uri.push_str("/");
    uri.push_str(database);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&uri)
        .await?;

    Ok(pool)
}

pub fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

pub fn serialize_dt<S>(dt: &chrono::DateTime<chrono::Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&chrono::DateTime::to_rfc3339(dt))
}

#[derive(Debug)]
pub struct ParseError;

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error parsing arg to type")
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {}
