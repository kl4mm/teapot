use serde::{Deserialize, Serialize, Serializer};
use sqlx::{types::chrono, FromRow, PgPool};
use uuid::Uuid;

use super::inventory::serialize_dt;

#[derive(Deserialize)]
pub struct OrderRequest {
    id: i64,
    quantity: i32,
}

#[derive(Serialize, FromRow)]
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

    fn build_get(filters: Vec<&str>, limit: &str, offset: &str) -> String {
        let mut sql = String::from("SELECT * FROM orders");

        // Match each filter and append to query
        let mut filterv = Vec::new();
        for filter in filters {
            match filter {
                "userId" => filterv.push("user_id = $1"),
                "id" => filterv.push("id = $2"),
                _ => {}
            }
        }

        sql.push_str(" WHERE ");
        sql.push_str(&filterv.join(" AND "));

        // Append LIMIT and OFFSET
        sql.push_str(" LIMIT ");
        sql.push_str(limit);
        sql.push_str(" OFFSET ");
        sql.push_str(offset);

        sql
    }

    pub async fn get(
        pool: &PgPool,
        id: Option<Uuid>,
        user_id: i64,
        filters: Vec<&str>,
        limit: &str,
        offset: &str,
    ) -> Result<Vec<Order>, sqlx::Error> {
        let sql = Self::build_get(filters, limit, offset);
        Ok(sqlx::query_as(&sql)
            .bind(user_id)
            .bind(id)
            .fetch_all(pool)
            .await?)
    }
}
