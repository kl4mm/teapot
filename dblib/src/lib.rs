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

#[macro_export]
macro_rules! bind {
    ( $args:ident, $query:ident, $( $x:expr => $t:ty ),* ) => {
        {
            for (column, arg) in $args {
                match column.as_str() {
                    $(
                        $x => {
                            let parsed: $t = arg.parse().unwrap();
                            $query = $query.bind(parsed);
                        }
                    )*
                    _ => {}
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {}
