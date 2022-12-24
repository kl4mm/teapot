use apilib::{set_response, App};
use dblib::shop::inventory::Inventory;
use hyper::{http::HeaderValue, Body, Method, Request, Response, StatusCode};
use sqlx::PgPool;
use std::{convert::Infallible, sync::Arc};

pub async fn handle(app: Arc<App>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    response
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    let (parts, mut _body) = req.into_parts();

    let response = match (parts.method, parts.uri.path()) {
        // (Method::POST, "/") => sign_up(&app.pool, &mut body, response).await,
        // (Method::POST, "/token") => token(&app.pool, &mut body, response).await,
        (Method::GET, "/inventory") => get_inventory(&app.pool, parts.uri.query(), response).await,
        (Method::POST, "/address") => todo!(),
        (Method::GET, "/address") => todo!(),
        (Method::POST, "/orders") => todo!(),
        (Method::GET, "/orders") => todo!(),
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            Ok(response)
        }
    };

    let response = match response {
        Ok(r) => r,
        Err(code) => set_response(Response::new(Body::empty()), code, None),
    };

    Ok(response)
}

async fn get_inventory(
    pool: &PgPool,
    query: Option<&str>,
    mut response: Response<Body>,
) -> Result<Response<Body>, StatusCode> {
    let query = apilib::parse_query(query);

    let mut filter = Vec::new();
    if let Some(&q) = query.get("inStock") {
        if q == "1" {
            filter.push("inStock")
        }
    }

    let id: Option<i64> = if let Some(id) = query.get("id") {
        filter.push("id");
        Some(id.parse().map_err(|_| StatusCode::BAD_REQUEST)?)
    } else {
        None
    };

    let sort = query.get("sort");
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

    let inventory = Inventory::get(&pool, id, filter, sort, limit.unwrap(), offset.unwrap())
        .await
        .map_err(|e| {
            log::debug!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let res = serde_json::to_string(&inventory).map_err(|e| {
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
    async fn test_get_inventory(_pool: sqlx::PgPool) -> sqlx::Result<()> {
        Ok(())
    }
}