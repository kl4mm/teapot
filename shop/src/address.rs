use std::collections::HashSet;

use dblib::shop::address::Address;
use hyper::{Body, Response, StatusCode};
use query::UrlQuery;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Either, PgPool};

#[derive(Deserialize)]
struct PostAddressRequest {
    #[serde(rename(deserialize = "userId"))]
    user_id: i64,
    #[serde(rename(deserialize = "firstName"))]
    first_name: String,
    #[serde(rename(deserialize = "lastName"))]
    last_name: String,
    #[serde(rename(deserialize = "address1"))]
    address_1: String,
    #[serde(rename(deserialize = "address2"))]
    address_2: String,
    postcode: String,
    city: String,
}

pub async fn post_address(
    pool: &PgPool,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let r: PostAddressRequest = serde_json::from_slice(&bytes).map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::UNPROCESSABLE_ENTITY, None)
    })?;

    let address = Address::new(
        pool,
        r.user_id,
        r.first_name,
        r.last_name,
        r.address_1,
        r.address_2,
        r.postcode,
        r.city,
    )
    .await
    .map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    let res = serde_json::to_string(&address).map_err(|e| {
        log::debug!("{}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, None)
    })?;

    *response.status_mut() = StatusCode::CREATED;
    *response.body_mut() = Body::from(res);

    Ok(response)
}

pub async fn get_address(
    pool: &PgPool,
    query: Option<&str>,
    mut response: Response<Body>,
) -> Result<Response<Body>, (StatusCode, Option<serde_json::Value>)> {
    let query = query.unwrap_or("");
    let allowed_fields = HashSet::from(["userId"]);
    let parsed = UrlQuery::new(query, &allowed_fields).map_err(|e| {
        log::debug!("{:?}", e);
        (
            StatusCode::BAD_REQUEST,
            Some(json!({ "message": "invalid query" })),
        )
    })?;

    if let Err(e) = parsed.check_required(vec!["userId"]) {
        Err((StatusCode::BAD_REQUEST, Some(json!({ "message": e }))))?
    }

    if let Err(e) = parsed.check_limit_and_offset() {
        Err((StatusCode::BAD_REQUEST, Some(json!({ "message": e }))))?
    }

    let addresses = Address::get(pool, parsed).await.map_err(|e| {
        log::debug!("{}", e);
        match e {
            Either::Right(_) => (StatusCode::BAD_REQUEST, None),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, None),
        }
    })?;

    let res = serde_json::to_string(&addresses).map_err(|e| {
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
    async fn test_post_address(_pool: sqlx::PgPool) -> sqlx::Result<()> {
        Ok(())
    }
}
