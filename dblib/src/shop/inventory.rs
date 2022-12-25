use serde::{Serialize, Serializer};
use sqlx::{types::chrono, FromRow, PgPool};

#[derive(Serialize, FromRow)]
pub struct Inventory {
    id: i64,
    name: String,
    price: i32,
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
    fn build_get(filters: Vec<&str>, sort: Option<&&str>, limit: &str, offset: &str) -> String {
        let mut sql = String::from("SELECT * FROM inventory");

        // Match each filter and append to query
        let mut filterv = Vec::new();
        for filter in filters {
            match filter {
                "inStock" => filterv.push("quantity > 0"),
                "id" => filterv.push("id = $1"),
                _ => {}
            }
        }

        // Only need WHERE if any filters were matched:
        if filterv.len() > 0 {
            sql.push_str(" WHERE ");
            sql.push_str(&filterv.join(" AND "));
        }

        // Match sort, can only be one of the following:
        if let Some(&sort) = sort {
            match sort {
                "price-asc" => sql.push_str(" ORDER BY price ASC"),
                "price-desc" => sql.push_str(" ORDER BY price DESC"),
                _ => {}
            }
        }

        // Append LIMIT and OFFSET
        sql.push_str(" LIMIT ");
        sql.push_str(limit);
        sql.push_str(" OFFSET ");
        sql.push_str(offset);

        sql
    }

    pub async fn get(
        pool: &PgPool,
        id: Option<i64>,
        filters: Vec<&str>,
        sort: Option<&&str>,
        limit: &str,
        offset: &str,
    ) -> Result<Vec<Inventory>, sqlx::Error> {
        let sql = Self::build_get(filters, sort, limit, offset);
        Ok(sqlx::query_as(&sql).bind(id).fetch_all(pool).await?)
    }
}

#[cfg(test)]
mod test {
    use super::Inventory;

    #[test]
    fn test_build_get_asc() {
        let filters = vec!["inStock", "id"];
        let sort = Some(&"price-asc");
        let limit = "10";
        let offset = "0";

        let sql = Inventory::build_get(filters, sort, limit, offset);

        assert_eq!(
            sql,
            "SELECT * FROM inventory WHERE quantity > 0 AND id = $1 ORDER BY price ASC LIMIT 10 OFFSET 0"
        );
    }

    #[test]
    fn test_build_get_desc() {
        let filters = vec!["inStock", "id"];
        let sort = Some(&"price-desc");
        let limit = "10";
        let offset = "0";

        let sql = Inventory::build_get(filters, sort, limit, offset);

        assert_eq!(
            sql,
            "SELECT * FROM inventory WHERE quantity > 0 AND id = $1 ORDER BY price DESC LIMIT 10 OFFSET 0"
        );
    }

    #[test]
    fn test_build_get_no_filter() {
        let filters = vec![];
        let sort = Some(&"price-desc");
        let limit = "10";
        let offset = "0";

        let sql = Inventory::build_get(filters, sort, limit, offset);

        assert_eq!(
            sql,
            "SELECT * FROM inventory ORDER BY price DESC LIMIT 10 OFFSET 0"
        );
    }
}
