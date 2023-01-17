use query::{
    sql::{Database::Postgres, QueryBuilder},
    sqlx_bind, UrlQuery,
};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::{types::chrono, Either, FromRow, PgPool};
use uuid::Uuid;

use crate::ParseError;

use super::inventory::serialize_dt;

#[derive(Deserialize)]
pub struct OrderRequest {
    id: i64,
    quantity: i32,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Order {
    #[serde(serialize_with = "serialize_uuid")]
    id: Uuid,
    #[serde(rename(serialize = "userId"))]
    user_id: i64,
    status: String,
    #[serde(rename(serialize = "addressId"))]
    address_id: i64,
    #[serde(serialize_with = "serialize_dt", rename(serialize = "createdAt"))]
    created_at: chrono::DateTime<chrono::Utc>,
    total: i64,
}

#[derive(Serialize, FromRow)]
pub struct OrderDetail {
    #[serde(serialize_with = "serialize_uuid")]
    id: Uuid,
    name: String,
    #[serde(rename(serialize = "imageUrl"))]
    image_url: String,
    status: String,
    quantity: i32,
    total: i32,
}

pub fn serialize_uuid<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&uuid.to_string())
}

impl Order {
    pub async fn new(
        pool: &PgPool,
        user_id: i64,
        address_id: i64,
        request: Vec<OrderRequest>,
    ) -> Result<Uuid, sqlx::Error> {
        let mut tx = pool.begin().await?;
        let uuid = Uuid::new_v4();
        sqlx::query("INSERT INTO orders (id, user_id, address_id) VALUES ($1, $2, $3)")
            .bind(&uuid)
            .bind(user_id)
            .bind(address_id)
            .execute(&mut tx)
            // Should rollback according to docs:
            .await?;
        for item in request {
            sqlx::query("INSERT INTO order_items VALUES ($1, $2, $3)")
                .bind(&uuid)
                .bind(item.id)
                .bind(item.quantity)
                .execute(&mut tx)
                // Should rollback according to docs:
                .await?;
            sqlx::query("UPDATE inventory SET quantity = quantity - $1 WHERE id = $2")
                .bind(item.quantity)
                .bind(item.id)
                .execute(&mut tx)
                .await?;
        }

        tx.commit().await?;

        Ok(uuid)
    }

    pub async fn get(
        pool: &PgPool,
        query: UrlQuery,
    ) -> Result<Vec<Order>, Either<sqlx::Error, ParseError>> {
        let (sql, args) = QueryBuilder::from_str(
            "SELECT orders.id, user_id, status, address_id, \
            orders.created_at, SUM(order_items.quantity * price) AS total FROM orders \
            JOIN order_items ON orders.id = order_items.order_id \
            JOIN inventory ON order_items.inventory_id = inventory.id",
            query,
            Postgres,
        )
        .map_columns([("id", "orders")].into())
        .build();

        let mut query = sqlx::query_as(&sql);

        sqlx_bind!(
            args => query,
            error: Either::Right(ParseError),
            "id" => Uuid,
            "userId" => i64
        );

        Ok(query.fetch_all(pool).await.map_err(|e| Either::Left(e))?)
    }
}

impl OrderDetail {
    pub async fn get(
        pool: &PgPool,
        query: UrlQuery,
    ) -> Result<Vec<OrderDetail>, Either<sqlx::Error, ParseError>> {
        let (sql, args) = QueryBuilder::from_str(
            "SELECT orders.id, name, image_url, status, order_items.quantity, \
	        price, order_items.quantity * price AS total FROM orders \
            JOIN order_items ON orders.id = order_items.order_id \
            JOIN inventory ON order_items.inventory_id = inventory.id",
            query,
            Postgres,
        )
        .map_columns([("id", "orders"), ("createdAt", "orders")].into())
        .build();

        let mut query = sqlx::query_as(&sql);

        sqlx_bind!(
            args => query,
            error: Either::Right(ParseError),
            "id" => Uuid,
            "userId" => i64
        );

        Ok(query.fetch_all(pool).await.map_err(|e| Either::Left(e))?)
    }
}
