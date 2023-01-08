use std::collections::HashSet;

use apilib::set_response;
use dblib::shop::inventory::Inventory;
use hyper::{Body, Response, StatusCode};
use query::query::Query;
use sqlx::PgPool;

pub async fn get_inventory(
    pool: &PgPool,
    query: Option<&str>,
    mut response: Response<Body>,
) -> Result<Response<Body>, StatusCode> {
    let query = query.unwrap_or("");
    let allowed_fields = HashSet::from(["quantity", "id", "price", "createdAt"]);
    let parsed = Query::new(query, &allowed_fields).map_err(|e| {
        log::debug!("{:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    if let Err(e) = parsed.check_limit_and_offset() {
        return Ok(set_response(
            response,
            StatusCode::BAD_REQUEST,
            // TODO: Would be better if Err was (StatusCode, Option<serde_json::Value>)
            Some(&serde_json::json!({ "message": e }).to_string()),
        ));
    }

    let inventory = Inventory::get(&pool, &parsed).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let res = serde_json::to_string(&inventory).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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
