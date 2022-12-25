use serde::Serialize;
use sqlx::{PgPool, Row};

#[derive(Serialize)]
pub struct Address {
    id: i64,
    #[serde(rename(serialize = "userId"))]
    user_id: i64,
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
        address_1: String,
        address_2: String,
        postcode: String,
        city: String,
    ) -> Result<Self, sqlx::Error> {
        let row = sqlx::query(
            "\
            INSERT INTO address (user_id, address_1, address_2, postcode, city) \
            VALUES ($1, $2, $3, $4, $5) \
            RETURNING id",
        )
        .bind(user_id)
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
            address_1,
            address_2,
            postcode,
            city,
        })
    }
}
