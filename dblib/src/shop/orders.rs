use query::{
    sql::{Database::Postgres, QueryBuilder},
    UrlQuery,
};
use serde::{Deserialize, Serialize, Serializer};
use sqlx::{types::chrono, FromRow, PgPool};
use uuid::Uuid;

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
}

// #[derive(Serialize, FromRow)]
// pub struct JoinedOrder {
//     #[serde(serialize_with = "serialize_uuid")]
//     id: Uuid,
//     #[serde(rename(serialize = "userId"))]
//     user_id: i64,
//     quantity: i32,
//     #[serde(rename(serialize = "inventoryId"))]
//     // Inventory columns:
//     name: String,
//     price: i32,
//     #[serde(rename(serialize = "imageUrl"))]
//     image_url: String,
//     #[serde(rename(serialize = "addressId"))]
//     address_id: i64,
// }

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

    pub async fn get(pool: &PgPool, query: UrlQuery) -> Result<Vec<Order>, sqlx::Error> {
        let (sql, fields) = QueryBuilder::from_str("SELECT * FROM orders", query, Postgres).build();
        let mut query = sqlx::query_as(&sql);

        for (field, param) in fields {
            match field.as_str() {
                "id" => {
                    let id: Uuid = param.parse().unwrap();
                    query = query.bind(id);
                }
                "userId" => {
                    let user_id: i64 = param.parse().unwrap();
                    query = query.bind(user_id);
                }
                _ => {}
            }
        }

        Ok(query.fetch_all(pool).await?)
    }
}
