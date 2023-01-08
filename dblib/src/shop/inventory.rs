use query::{query::Query, sql};
use serde::{Serialize, Serializer};
use sqlx::{types::chrono, FromRow, PgPool};

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
    pub async fn get(pool: &PgPool, query: &Query) -> Result<Vec<Inventory>, sqlx::Error> {
        let (sql, fields) = sql::gen_psql(&query, "inventory", vec!["*"], vec![]);
        dbg!(&sql);
        dbg!(&fields);

        let mut query = sqlx::query_as(&sql);
        for (field, param) in fields {
            match field {
                "id" => {
                    let id: i64 = param.parse().unwrap();
                    query = query.bind(id);
                }
                "quantity" => {
                    let quantity: i32 = param.parse().unwrap();
                    query = query.bind(quantity);
                }
                "price" => {
                    let price: i32 = param.parse().unwrap();
                    query = query.bind(price);
                }
                "createdAt" => {
                    let created_at: String = param.parse().unwrap();
                    query = query.bind(created_at);
                }
                _ => {}
            }
        }

        Ok(query.fetch_all(pool).await?)
    }
}
