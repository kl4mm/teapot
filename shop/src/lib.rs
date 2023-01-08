pub mod address;
pub mod cart;
pub mod inventory;
pub mod orders;

use address::{get_address, post_address};
use apilib::{set_response, App};
use cart::{delete_cart, get_cart, patch_cart, post_cart};
use hyper::{http::HeaderValue, Body, Method, Request, Response, StatusCode};
use inventory::get_inventory;
use orders::{get_orders, post_orders};
use std::{convert::Infallible, sync::Arc};

pub async fn handle(app: Arc<App>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    response
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    let (parts, mut body) = req.into_parts();

    let response = match (parts.method, parts.uri.path()) {
        (Method::GET, "/inventory") => get_inventory(&app.pool, parts.uri.query(), response).await,
        (Method::POST, "/address") => post_address(&app.pool, &mut body, response).await,
        (Method::GET, "/address") => get_address(&app.pool, parts.uri.query(), response).await,
        (Method::GET, "/cart") => {
            get_cart(app.redis.as_ref().unwrap(), &parts.headers, response).await
        }
        (Method::POST, "/cart") => {
            post_cart(
                app.redis.as_ref().unwrap(),
                &parts.headers,
                &mut body,
                response,
            )
            .await
        }
        (Method::PATCH, "/cart") => {
            patch_cart(
                app.redis.as_ref().unwrap(),
                &parts.headers,
                &mut body,
                response,
            )
            .await
        }
        (Method::DELETE, "/cart") => {
            delete_cart(
                app.redis.as_ref().unwrap(),
                &parts.headers,
                &mut body,
                response,
            )
            .await
        }
        (Method::POST, "/orders") => post_orders(&app.pool, &mut body, response).await,
        (Method::GET, "/orders") => get_orders(&app.pool, parts.uri.query(), response).await,
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
