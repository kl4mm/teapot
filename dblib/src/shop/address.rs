use query::{
    sql::{Database::Postgres, QueryBuilder},
    sqlx_bind, UrlQuery,
};
use serde::Serialize;
use sqlx::{Either, FromRow, PgPool, Row};

use crate::ParseError;

#[derive(Serialize, FromRow)]
pub struct Address {
    id: i64,
    #[serde(rename(serialize = "userId"))]
    user_id: i64,
    #[serde(rename(serialize = "firstName"))]
    first_name: String,
    #[serde(rename(serialize = "lastName"))]
    last_name: String,
    #[serde(rename(serialize = "address1"))]
    address_1: String,
    #[serde(rename(serialize = "address2"))]
    address_2: String,
    postcode: String,
    city: String,
}

impl Address {
    pub async fn new(
        pool: &PgPool,
        user_id: i64,
        first_name: String,
        last_name: String,
        address_1: String,
        address_2: String,
        postcode: String,
        city: String,
    ) -> Result<Self, sqlx::Error> {
        let row = sqlx::query(
            "\
            INSERT INTO address (user_id, first_name, last_name, address_1, address_2, postcode, city) \
            VALUES ($1, $2, $3, $4, $5, $6, $7) \
            RETURNING id",
        )
        .bind(user_id)
        .bind(&first_name)
        .bind(&last_name)
        .bind(&address_1)
        .bind(&address_2)
        .bind(&postcode)
        .bind(&city)
        .fetch_one(pool)
        .await?;

        let id = row.try_get("id")?;

        Ok(Self {
            id,
            user_id,
            first_name,
            last_name,
            address_1,
            address_2,
            postcode,
            city,
        })
    }

    pub async fn get(
        pool: &PgPool,
        query: UrlQuery,
    ) -> Result<Vec<Self>, Either<sqlx::Error, ParseError>> {
        let (sql, args) = QueryBuilder::from_str("SELECT * FROM address", query, Postgres).build();
        let mut query = sqlx::query_as(&sql);

        sqlx_bind!(
            args => query,
            error: Either::Right(ParseError),
            "userId" => i64
        );

        Ok(query.fetch_all(pool).await.map_err(|e| Either::Left(e))?)
    }
}
