use std::collections::HashSet;

use hyper::{Body, HeaderMap, Response, StatusCode};
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use towerlib::session::get_session;

fn cart_key(session_id: &str) -> String {
    let mut key = String::from("cart:");
    key.push_str(session_id);
    key
}

#[derive(Serialize, Deserialize)]
struct CartItem {
    id: i64,
    name: String,
    price: i32,
    #[serde(rename(deserialize = "imageUrl", serialize = "imageUrl"))]
    image_url: String,
    quantity: i32,
}

// To share with post_orders:
pub async fn get_cart_hashset(
    redis: &RedisClient,
    headers: &HeaderMap,
) -> Result<HashSet<String>, (StatusCode, Option<serde_json::Value>)> {
    let session = get_session(&headers)?;

    let mut con = redis.get_async_connection().await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let key = cart_key(session);

    let cart: HashSet<String> = con.smembers(key).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    Ok(cart)
}

pub async fn get_cart(
    redis: &RedisClient,
    headers: &HeaderMap,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let cart = get_cart_hashset(redis, headers).await?;

    let mut res = Vec::new();

    for item in &cart {
        let i: CartItem = serde_json::from_str(&item).map_err(|e| {
            log::error!("{}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, None)
        })?;
        res.push(i)
    }

    let res = serde_json::to_string(&res).map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    *response.status_mut() = StatusCode::OK;
    *response.body_mut() = Body::from(res);

    Ok(response)
}

pub async fn post_cart(
    redis: &RedisClient,
    headers: &HeaderMap,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let session = get_session(&headers)?;

    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    // Check if valid request:
    let json = serde_json::from_slice::<CartItem>(&bytes).map_err(|e| {
        log::error!("{}", e);
        (StatusCode::UNPROCESSABLE_ENTITY, None)
    })?;

    let body_str = serde_json::to_string(&json).unwrap();

    let mut con = redis.get_async_connection().await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let key = cart_key(session);

    con.sadd(key, body_str).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    *response.status_mut() = StatusCode::OK;
    *response.body_mut() = Body::from("{\"message\": \"success\"}");

    Ok(response)
}

#[derive(Deserialize)]
struct PatchCartRequest {
    old: CartItem,
    new: CartItem,
}

pub async fn patch_cart(
    redis: &RedisClient,
    headers: &HeaderMap,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let session = get_session(&headers)?;

    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let r: PatchCartRequest = serde_json::from_slice(&bytes).map_err(|e| {
        log::error!("{}", e);
        (StatusCode::UNPROCESSABLE_ENTITY, None)
    })?;

    let old = serde_json::to_string(&r.old).map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let new = serde_json::to_string(&r.new).map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let mut con = redis.get_async_connection().await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let key = cart_key(session);

    // TODO: redis::transaction doesn't take async connection?
    con.srem(&key, old).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    con.sadd(&key, new).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    *response.status_mut() = StatusCode::OK;
    *response.body_mut() = Body::from("{\"message\": \"success\"}");

    Ok(response)
}

pub async fn delete_cart(
    redis: &RedisClient,
    headers: &HeaderMap,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let session = get_session(&headers)?;

    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let json = serde_json::from_slice::<CartItem>(&bytes).map_err(|e| {
        log::error!("{}", e);
        (StatusCode::UNPROCESSABLE_ENTITY, None)
    })?;

    let body_str = serde_json::to_string(&json).unwrap();

    let mut con = redis.get_async_connection().await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let key = cart_key(session);

    con.srem(&key, body_str).await.map_err(|e| {
        log::error!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    *response.status_mut() = StatusCode::OK;
    *response.body_mut() = Body::from("{\"message\": \"success\"}");

    Ok(response)
}
