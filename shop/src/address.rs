use dblib::shop::address::Address;
use hyper::{Body, Response, StatusCode};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
struct PostAddressRequest {
    #[serde(rename(deserialize = "userId"))]
    user_id: i64,
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
) -> Result<Response<Body>, StatusCode> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let r: PostAddressRequest = serde_json::from_slice(&bytes).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let address = Address::new(
        pool,
        r.user_id,
        r.address_1,
        r.address_2,
        r.postcode,
        r.city,
    )
    .await
    .map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let res = serde_json::to_string(&address).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    *response.status_mut() = StatusCode::CREATED;
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
