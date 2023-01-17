use std::collections::HashSet;

use dblib::shop::orders::{Order, OrderDetail, OrderRequest};
use hyper::{Body, Response, StatusCode};
use query::UrlQuery;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Either, PgPool};

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
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let r: PostOrdersRequest = serde_json::from_slice(&bytes).map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::UNPROCESSABLE_ENTITY, None)
    })?;

    let order_id = Order::new(pool, r.user_id, r.address_id, r.items)
        .await
        .map_err(|e| {
            log::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, None)
        })?;

    let res = serde_json::json!({ "orderId": order_id.to_string() });
    let res = serde_json::to_string(&res).map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    *response.status_mut() = StatusCode::CREATED;
    *response.body_mut() = Body::from(res);

    Ok(response)
}

pub async fn get_orders(
    pool: &PgPool,
    query: Option<&str>,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let query = query.unwrap_or("");
    let allowed_fields = HashSet::from(["userId", "id", "createdAt"]);
    let mut parsed = UrlQuery::new(query, &allowed_fields).map_err(|e| {
        log::debug!("{:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Some(json!({ "message": "invalid query" })),
        )
    })?;

    if parsed.params.is_empty() {
        Err((StatusCode::BAD_REQUEST, None))?
    }

    if let Err(e) = parsed.check_limit_and_offset() {
        Err((StatusCode::BAD_REQUEST, Some(json!({ "message": e }))))?
    }

    let res: String;
    match parsed.params.get("id") {
        Some(_) => {
            *parsed.group_mut() = None;

            let orders = OrderDetail::get(pool, parsed).await.map_err(|e| {
                log::error!("{}", e);
                match e {
                    Either::Right(_) => (StatusCode::BAD_REQUEST, None),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
                }
            })?;

            res = serde_json::to_string(&orders).map_err(|e| {
                log::debug!("{}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, None)
            })?;
        }
        None => {
            // This shouldn't be required in the request
            *parsed.group_mut() = Some("id".into());

            let orders = Order::get(pool, parsed).await.map_err(|e| {
                log::error!("{}", e);
                match e {
                    Either::Right(_) => (StatusCode::BAD_REQUEST, None),
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
                }
            })?;

            res = serde_json::to_string(&orders).map_err(|e| {
                log::debug!("{}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, None)
            })?;
        }
    };

    *response.status_mut() = StatusCode::OK;
    *response.body_mut() = Body::from(res);

    Ok(response)
}
