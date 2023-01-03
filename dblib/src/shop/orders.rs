use serde::{Deserialize, Serialize, Serializer};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct OrderRequest {
    id: i64,
    quantity: i32,
}

#[derive(Serialize)]
pub struct Order {
    #[serde(serialize_with = "serialize_uuid")]
    id: Uuid,
    #[serde(rename(serialize = "userId"))]
    user_id: i64,
    #[serde(rename(serialize = "inventoryId"))]
    inventory_id: i64,
    quantity: i32,
    #[serde(rename(serialize = "addressId"))]
    address_id: i64,
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
        for item in request {
            sqlx::query("INSERT INTO orders VALUES ($1, $2, $3, $4, $5)")
                .bind(&uuid)
                .bind(user_id)
                .bind(&item.id)
                .bind(item.quantity)
                .bind(address_id)
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
}
