use query::{
    sql::{Database::Postgres, QueryBuilder},
    sqlx_bind, UrlQuery,
};
use serde::{Serialize, Serializer};
use sqlx::{types::chrono, Either, FromRow, PgPool};

use crate::ParseError;

#[derive(Serialize, FromRow)]
pub struct Inventory {
    id: i64,
    name: String,
    price: i32,
    quantity: i32,
    #[serde(rename(serialize = "imageUrl"))]
    image_url: String,
    description: String,
    #[serde(serialize_with = "serialize_dt", rename(serialize = "createdAt"))]
    created_at: chrono::DateTime<chrono::Utc>,
}

pub fn serialize_dt<S>(dt: &chrono::DateTime<chrono::Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&chrono::DateTime::to_rfc3339(dt))
}

impl Inventory {
    pub async fn get(
        pool: &PgPool,
        query: UrlQuery,
    ) -> Result<Vec<Inventory>, Either<sqlx::Error, ParseError>> {
        let (sql, args) =
            QueryBuilder::from_str("SELECT * FROM inventory", query, Postgres).build();

        let mut query = sqlx::query_as(&sql);

        sqlx_bind! (
            args => query,
            error: Either::Right(ParseError),
            "id" => i64,
            "quantity" => i32,
            "price" => i32,
            "createdAt" => String
        );

        Ok(query.fetch_all(pool).await.map_err(|e| Either::Left(e))?)
    }
}
