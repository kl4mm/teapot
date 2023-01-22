use convert_case::Case;
use query::{sql::QueryBuilder, sqlx_bind, UrlQuery};
use serde::Serialize;
use sqlx::{types::chrono, Either, FromRow, PgPool};

use crate::{serialize_dt, ParseError};

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

impl Inventory {
    pub async fn get(
        pool: &PgPool,
        query: UrlQuery,
    ) -> Result<Vec<Inventory>, Either<sqlx::Error, ParseError>> {
        let (sql, args) = QueryBuilder::from_str("SELECT * FROM inventory", query)
            .convert_case(Case::Snake)
            .build();

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
