use dblib::shop::inventory::Inventory;
use hyper::{Body, Response, StatusCode};
use query::UrlQuery;
use serde_json::json;
use sqlx::{Either, PgPool};

pub async fn get_inventory(
    pool: &PgPool,
    query: Option<&str>,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let query = query.unwrap_or("");
    let parsed = UrlQuery::new(query, ["quantity", "id", "price", "createdAt"]).map_err(|e| {
        log::debug!("{:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Some(json!({ "message": "invalid query" })),
        )
    })?;

    if let Err(e) = parsed.check_limit_and_offset() {
        Err((StatusCode::BAD_REQUEST, Some(json!({ "message": e }))))?
    }

    let inventory = Inventory::get(pool, parsed).await.map_err(|e| {
        log::debug!("{}", e);
        match e {
            Either::Right(_) => (StatusCode::BAD_REQUEST, None),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
        }
    })?;

    let res = serde_json::to_string(&inventory).map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    *response.status_mut() = StatusCode::OK;
    *response.body_mut() = Body::from(res);

    Ok(response)
}

#[cfg(test)]
mod test {
    #[sqlx::test(fixtures("shop"))]
    async fn test_get_inventory(_pool: sqlx::PgPool) -> sqlx::Result<()> {
        Ok(())
    }
}
