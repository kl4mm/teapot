use apilib::set_response;
use dblib::shop::orders::{Order, OrderRequest};
use hyper::{Body, Response, StatusCode};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

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
    let query = apilib::parse_query(query);

    let mut filter = Vec::new();

    let user_id = match query.get("userId") {
        Some(&q) => {
            filter.push(q);
            q
        }
        None => {
            return Ok(set_response(
                response,
                StatusCode::BAD_REQUEST,
                Some("{\"message\": \"userId is required\"}"),
            ));
        }
    };

    let id: Option<Uuid> = if let Some(id) = query.get("id") {
        filter.push("id");
        Some(id.parse().map_err(|_| StatusCode::BAD_REQUEST)?)
    } else {
        None
    };

    let limit = query.get("limit");
    let offset = query.get("offset");

    if let None = limit {
        return Ok(set_response(
            response,
            StatusCode::BAD_REQUEST,
            Some("{\"message\": \"limit is required\"}"),
        ));
    };

    if let None = offset {
        return Ok(set_response(
            response,
            StatusCode::BAD_REQUEST,
            Some("{\"message\": \"offset is required\"}"),
        ));
    };

    todo!()
}
