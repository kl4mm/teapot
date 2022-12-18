use dblib::users::users::{Password, User};
use hyper::{http::HeaderValue, Body, Method, Request, Response, StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, Pool, Postgres};
use std::{convert::Infallible, sync::Arc};

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
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(b) => b,
        Err(e) => {
            dbg!(e);
            response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
            return Ok(response);
        }
    };

    let r: SignupRequest = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(e) => {
            dbg!(e);
            response = set_response(response, StatusCode::UNPROCESSABLE_ENTITY, None);
            return Ok(response);
        }
    };

    let hash = match Password::hash(&r.password) {
        Ok(p) => p,
        Err(e) => {
            dbg!(e);
            response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
            return Ok(response);
        }
    };

    let user = match User::new(pool, r.first_name, r.last_name, r.email, hash).await {
        Ok(u) => u,
        Err(e) => match e {
            sqlx::Error::Database(err) => {
                if err.code().is_some() && err.code().unwrap().contains("23505") {
                    // Unique violation:
                    response = set_response(
                        response,
                        StatusCode::BAD_REQUEST,
                        Some(r#"{"message": "Email has already signed up. Please log in."}"#),
                    );
                    return Ok(response);
                } else {
                    // Other db error
                    dbg!(err);
                    response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
                    return Ok(response);
                };
            }
            e => {
                dbg!(e);
                response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
                return Ok(response);
            }
        },
    };

    let res = match serde_json::to_string(&user) {
        Ok(r) => r,
        Err(e) => {
            dbg!(e);
            response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
            return Ok(response);
        }
    };

    *response.status_mut() = StatusCode::CREATED;
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
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(b) => b,
        Err(e) => {
            dbg!(e);
            response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
            return Ok(response);
        }
    };

    let r: TokenRequest = match serde_json::from_slice(&bytes) {
        Ok(r) => r,
        Err(e) => {
            dbg!(e);
            response = set_response(response, StatusCode::UNPROCESSABLE_ENTITY, None);
            return Ok(response);
        }
    };

    let hash = match Password::hash(&r.password) {
        Ok(p) => p,
        Err(e) => {
            dbg!(e);
            response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
            return Ok(response);
        }
    };

    let hash_bytes = hash.into_bytes();
    let user = match User::from_email_and_password(pool, r.email, &hash_bytes).await {
        Ok(u) => u,
        Err(e) => match e {
            sqlx::Error::RowNotFound => {
                response = set_response(
                    response,
                    StatusCode::UNAUTHORIZED,
                    Some(r#"{"message": "Email or password incorrect"}"#),
                );
                return Ok(response);
            }
            _ => {
                // Other db error
                dbg!(e);
                response = set_response(response, StatusCode::INTERNAL_SERVER_ERROR, None);
                return Ok(response);
            }
        },
    };

    let res = serde_json::to_string(&user).unwrap();
    *response.body_mut() = Body::from(res);

    Ok(response)
}

const INTERNAL_SERVER_ERROR: &str = r#"{"message": "Internal Server Error"}"#;
const UNPROCESSABLE_ENTITY: &str = r#"{"message": "Unprocessable Entity"}"#;

fn set_response(
    mut response: Response<Body>,
    code: StatusCode,
    message: Option<&str>,
) -> Response<Body> {
    *response.status_mut() = code;

    let body = match message {
        Some(m) => Body::from(m.to_owned()),
        None => match code {
            // Messages for each code:
            StatusCode::INTERNAL_SERVER_ERROR => Body::from(INTERNAL_SERVER_ERROR),
            StatusCode::UNPROCESSABLE_ENTITY => Body::from(UNPROCESSABLE_ENTITY),
            _ => Body::empty(),
        },
    };

    *response.body_mut() = body;

    response
}
