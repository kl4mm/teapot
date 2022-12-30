use std::collections::HashSet;

use hyper::{Body, HeaderMap, Response, StatusCode};
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct CartItem {
    id: i64,
    name: String,
    price: i32,
    #[serde(rename(deserialize = "imageUrl"))]
    image_url: String,
    quantity: i32,
}

pub async fn get_cart(
    redis: &RedisClient,
    headers: &HeaderMap,
    mut response: Response<Body>,
) -> Result<Response<Body>, StatusCode> {
    // TODO: session middleware
    let session = match headers.get("Session-ID") {
        Some(s) => s.to_str().map_err(|e| {
            log::debug!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
        // Theres was a problem assigning a session ID
        // in the middleware:
        None => Err(StatusCode::INTERNAL_SERVER_ERROR)?,
    };

    let mut con = redis.get_async_connection().await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut key = String::from("cart:");
    key.push_str(session);

    let cart: HashSet<String> = con.smembers(key).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut res = Vec::new();

    for item in &cart {
        let i: CartItem = serde_json::from_str(&item).map_err(|e| {
            log::debug!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        res.push(i)
    }

    let res = serde_json::to_string(&res).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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
) -> Result<Response<Body>, StatusCode> {
    // TODO: session middleware
    let session = match headers.get("Session-ID") {
        Some(s) => s.to_str().map_err(|e| {
            log::debug!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
        // Theres was a problem assigning a session ID
        // in the middleware:
        None => Err(StatusCode::INTERNAL_SERVER_ERROR)?,
    };

    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Check if valid request:
    serde_json::from_slice::<CartItem>(&bytes).map_err(|e| {
        log::error!("{}", e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let body_str = std::str::from_utf8(&bytes).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::BAD_REQUEST
    })?;

    let mut con = redis.get_async_connection().await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut key = String::from("cart:");
    key.push_str(session);

    con.sadd(key, body_str).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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
) -> Result<Response<Body>, StatusCode> {
    todo!()
}

pub async fn delete_cart(
    redis: &RedisClient,
    headers: &HeaderMap,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, StatusCode> {
    todo!()
}
