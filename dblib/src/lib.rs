use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

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
