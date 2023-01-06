use apilib::set_response;
use dblib::shop::address::Address;
use hyper::{Body, Response, StatusCode};
use serde::Deserialize;
use sqlx::PgPool;

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

pub async fn get_address(
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
            ))
        }
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

    let addresses = Address::get(pool, user_id, filter, limit.unwrap(), offset.unwrap())
        .await
        .map_err(|e| {
            log::debug!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let res = serde_json::to_string(&addresses).map_err(|e| {
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
    async fn test_post_address(_pool: sqlx::PgPool) -> sqlx::Result<()> {
        Ok(())
    }
}
