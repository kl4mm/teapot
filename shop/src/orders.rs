use dblib::shop::orders::{Order, OrderRequest};
use hyper::{Body, Response, StatusCode};
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
