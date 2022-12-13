use std::{convert::Infallible, sync::Arc};

use dblib::users::users::{Password, User};
use hyper::{http::HeaderValue, Body, Method, Request, Response, StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, Pool, Postgres};

pub struct App {
    pool: Pool<Postgres>,
}

impl App {
    pub fn new(pool: Pool<Postgres>) -> Arc<Self> {
        Arc::new(Self { pool })
    }
}

pub async fn handle(app: Arc<App>, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());
    response
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    let (parts, mut body) = req.into_parts();

    match (parts.method, parts.uri.path()) {
        (Method::POST, "/") => response = sign_up(&app.pool, &mut body, response).await.unwrap(),
        (Method::POST, "/token") => response = token(&app.pool, &mut body, response).await.unwrap(),
        _ => *response.status_mut() = StatusCode::NOT_FOUND,
    }

    Ok(response)
}

#[derive(Deserialize)]
struct SignupRequest {
    #[serde(rename(deserialize = "firstName", serialize = "firstName"))]
    first_name: String,
    #[serde(rename(deserialize = "lastName", serialize = "lastName"))]
    last_name: String,
    email: String,
    password: String,
}

async fn sign_up(
    pool: &PgPool,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, Infallible> {
    let bytes = hyper::body::to_bytes(body).await.unwrap();
    let r: SignupRequest = serde_json::from_slice(&bytes).unwrap();

    let password = Password::hash(&r.password).unwrap();
    let user = match User::new(pool, r.first_name, r.last_name, r.email, password).await {
        Ok(u) => u,
        Err(e) => match e {
            sqlx::Error::Database(err) => {
                if err.code().is_some() && err.code().unwrap().contains("23505") {
                    // Unique violation:
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    *response.body_mut() =
                        Body::from(r#"{"message": "Email has already signed up. Please log in."}"#);
                    return Ok(response);
                } else {
                    // Other db error
                    dbg!(err);
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    *response.body_mut() = Body::from(r#"{{"message": "Internal Server Error"}}"#);
                    return Ok(response);
                };
            }
            e => {
                dbg!(e);
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                *response.body_mut() = Body::from(r#"{{"message": "Internal Server Error"}}"#);
                return Ok(response);
            }
        },
    };

    let res = serde_json::to_string(&user).unwrap();
    *response.body_mut() = Body::from(res);

    Ok(response)
}

#[derive(Deserialize)]
struct TokenRequest {
    email: String,
    password: String,
}

async fn token(
    pool: &PgPool,
    body: &mut Body,
    mut response: Response<Body>,
) -> Result<Response<Body>, Infallible> {
    let bytes = hyper::body::to_bytes(body).await.unwrap();
    let r: TokenRequest = serde_json::from_slice(&bytes).unwrap();

    let hash = Password::hash(&r.password).unwrap();
    let hash_bytes = hash.into_bytes();
    let user = match User::from_email_and_password(pool, r.email, &hash_bytes).await {
        Ok(u) => u,
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                *response.status_mut() = StatusCode::BAD_REQUEST;
                *response.body_mut() =
                    Body::from(r#"{{"message": "Email or password incorrect"}}"#);
                return Ok(response);
            }
            _ => {
                // Other db error
                dbg!(e);
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                *response.body_mut() = Body::from(r#"{{"message": "Internal Server Error"}}"#);
                return Ok(response);
            }
        },
    };

    let res = serde_json::to_string(&user).unwrap();
    *response.body_mut() = Body::from(res);

    Ok(response)
}
