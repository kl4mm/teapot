pub mod address;
pub mod inventory;
pub mod orders;

use address::post_address;
use apilib::{set_response, App};
use hyper::{http::HeaderValue, Body, Method, Request, Response, StatusCode};
use inventory::get_inventory;
use std::{convert::Infallible, sync::Arc};

pub async fn handle(app: Arc<App>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    response
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    let (parts, mut body) = req.into_parts();

    let response = match (parts.method, parts.uri.path()) {
        // (Method::POST, "/") => sign_up(&app.pool, &mut body, response).await,
        // (Method::POST, "/token") => token(&app.pool, &mut body, response).await,
        (Method::GET, "/inventory") => get_inventory(&app.pool, parts.uri.query(), response).await,
        (Method::POST, "/address") => post_address(&app.pool, &mut body, response).await,
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
