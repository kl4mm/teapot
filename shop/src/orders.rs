use std::collections::HashSet;

use apilib::set_response;
use dblib::shop::orders::{Order, OrderRequest};
use hyper::{Body, Response, StatusCode};
use query::query::Query;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
struct PostOrdersRequest {
    #[serde(rename(deserialize = "userId"))]
    user_id: i64,
    #[serde(rename(deserialize = "addressId"))]
    address_id: i64,
    items: Vec<OrderRequest>,
}

pub async fn post_orders(
    pool: &PgPool,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, StatusCode> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let r: PostOrdersRequest = serde_json::from_slice(&bytes).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let order_id = Order::new(pool, r.user_id, r.address_id, r.items)
        .await
        .map_err(|e| {
            log::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let res = serde_json::json!({ "orderId": order_id.to_string() });
    let res = serde_json::to_string(&res).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    *response.status_mut() = StatusCode::CREATED;
    *response.body_mut() = Body::from(res);

    Ok(response)
}

pub async fn get_orders(
    pool: &PgPool,
    query: Option<&str>,
    mut response: Response<Body>,
) -> Result<Response<Body>, StatusCode> {
    let query = query.unwrap_or("");
    let allowed_fields = HashSet::from(["userId", "id"]);
    let parsed = Query::new(query, &allowed_fields).map_err(|e| {
        log::debug!("{:?}", e);
        StatusCode::BAD_REQUEST
    })?;

    if let Err(e) = parsed.check_valid(vec!["userId"]) {
        return Ok(set_response(
            response,
            StatusCode::BAD_REQUEST,
            // TODO: Would be better if Err was (StatusCode, Option<serde_json::Value>)
            Some(&serde_json::json!({ "message": e }).to_string()),
        ));
    }

    if let Err(e) = parsed.check_limit_and_offset() {
        return Ok(set_response(
            response,
            StatusCode::BAD_REQUEST,
            // TODO: Would be better if Err was (StatusCode, Option<serde_json::Value>)
            Some(&serde_json::json!({ "message": e }).to_string()),
        ));
    }

    let orders = Order::get(pool, &parsed).await.map_err(|e| {
        log::error!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let res = serde_json::to_string(&orders).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    *response.status_mut() = StatusCode::OK;
    *response.body_mut() = Body::from(res);

    Ok(response)
}
