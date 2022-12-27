use serde::Serialize;
use sqlx::{FromRow, PgPool, Row};

#[derive(Serialize, FromRow)]
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

    fn build_get(filters: Vec<&str>, limit: &str, offset: &str) -> String {
        let mut sql = String::from("SELECT * FROM address");

        let mut filterv = Vec::new();
        for filter in filters {
            match filter {
                "userId" => filterv.push("user_id = $1"),
                _ => {}
            }
        }

        if filterv.len() > 0 {
            sql.push_str(" WHERE ");
            sql.push_str(&filterv.join(" AND "));
        }

        sql.push_str(" LIMIT ");
        sql.push_str(limit);
        sql.push_str(" OFFSET ");
        sql.push_str(offset);

        sql
    }

    pub async fn get(
        pool: &PgPool,
        user_id: &str,
        filters: Vec<&str>,
        limit: &str,
        offset: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let sql = Self::build_get(filters, limit, offset);
        Ok(sqlx::query_as(&sql).bind(user_id).fetch_all(pool).await?)
    }
}

#[cfg(test)]
mod test {
    use super::Address;

    #[test]
    fn test_build_get() {
        let filters = vec!["userId"];
        let limit = "10";
        let offset = "0";

        let sql = Address::build_get(filters, limit, offset);

        assert_eq!(
            sql,
            "SELECT * FROM address WHERE user_id = $1 LIMIT 10 OFFSET 0"
        );
    }

    #[test]
    fn test_build_get_no_filter() {
        let filters = vec![];
        let limit = "10";
        let offset = "0";

        let sql = Address::build_get(filters, limit, offset);

        assert_eq!(sql, "SELECT * FROM address LIMIT 10 OFFSET 0");
    }
}
