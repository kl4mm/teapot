use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;

pub mod shop;
pub mod users;

pub async fn connect() -> Result<PgPool, sqlx::Error> {
    let uri = env::var("DATABASE_URI").unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&uri)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {}
