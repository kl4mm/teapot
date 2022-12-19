use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub mod shop;
pub mod users;

pub async fn connect(database: &str) -> Result<PgPool, sqlx::Error> {
    let mut uri = env::var("DATABASE_URL").expect("DATABASE_URI not present");
    uri.push_str("/");
    uri.push_str(database);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&uri)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {}
