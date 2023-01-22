use std::collections::HashSet;

use dblib::shop::orders::{Order, OrderDetail, OrderRequest};
use hyper::{Body, HeaderMap, Response, StatusCode};
use query::UrlQuery;
use redis::Client as RedisClient;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Either, PgPool};

use crate::cart;

#[derive(Deserialize)]
struct PostOrdersRequestV2 {
    #[serde(rename(deserialize = "userId"))]
    user_id: i64,
    #[serde(rename(deserialize = "addressId"))]
    address_id: i64,
}

pub async fn post_orders(
    pool: &PgPool,
    redis: &RedisClient,
    body: &mut Body,
    headers: &HeaderMap,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let r: PostOrdersRequestV2 = serde_json::from_slice(&bytes).map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::UNPROCESSABLE_ENTITY, None)
    })?;

    let cart = cart::get_cart_hashset(redis, headers).await?;

    let mut request = Vec::new();
    for item in &cart {
        let i: OrderRequest = serde_json::from_str(&item).map_err(|e| {
            log::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, None)
        })?;

        request.push(i);
    }

    let order_id = Order::new(pool, r.user_id, r.address_id, request)
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
