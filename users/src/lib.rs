pub mod token;

use apilib::set_response;
use dblib::users::users::{Password, User};
use hyper::{http::HeaderValue, Body, Method, Request, Response, StatusCode};
use serde::Deserialize;
use sqlx::{PgPool, Pool, Postgres};
use std::{convert::Infallible, sync::Arc};
use token::gen_token;

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

    let response = match (parts.method, parts.uri.path()) {
        (Method::POST, "/") => sign_up(&app.pool, &mut body, response).await,
        (Method::POST, "/token") => token(&app.pool, &mut body, response).await,
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
) -> Result<Response<Body>, StatusCode> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let r: SignupRequest = serde_json::from_slice(&bytes).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let hash = Password::hash(&r.password).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let user = match User::new(pool, r.first_name, r.last_name, r.email, hash).await {
        Ok(u) => u,
        Err(e) => match e {
            sqlx::Error::Database(e) => {
                if e.code().is_some() && e.code().unwrap().contains("23505") {
                    // Unique violation:
                    response = set_response(
                        response,
                        StatusCode::CONFLICT,
                        Some(r#"{"message": "Email has already signed up. Please log in."}"#),
                    );
                    return Ok(response);
                };
                // Other db error
                log::debug!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
            e => {
                log::debug!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    let res = serde_json::to_string(&user).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
) -> Result<Response<Body>, StatusCode> {
    let bytes = hyper::body::to_bytes(body).await.map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let r: TokenRequest = serde_json::from_slice(&bytes).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let hash = Password::hash(&r.password).map_err(|e| {
        log::debug!("{}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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
                log::debug!("{}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    };

    let token = gen_token(&user).map_err(|e| {
        log::debug!("{:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let res = serde_json::to_string(&user).unwrap();
    *response.body_mut() = Body::from(res);
    response.headers_mut().insert("token", token);

    Ok(response)
}

#[cfg(test)]
mod test {
    use hyper::{Body, Response, StatusCode};

    use super::{sign_up, token, App};

    #[sqlx::test(fixtures("users"))]
    async fn test_sign_up(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let app = App::new(pool);

        let mut body = Body::from(
            "\
{
    \"firstName\": \"bob\",
    \"lastName\": \"smith\",
    \"email\": \"bob@mail.com\",
    \"password\": \"password123\"
}",
        );

        let response = Response::new(Body::empty());

        let res = sign_up(&app.pool, &mut body, response).await.unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);

        Ok(())
    }

    #[sqlx::test(fixtures("users"))]
    async fn test_token(pool: sqlx::PgPool) -> sqlx::Result<()> {
        let app = App::new(pool);

        let mut body = Body::from(
            "\
{
    \"email\": \"bob@smith.com\",
    \"password\": \"password\"
}",
        );

        let response = Response::new(Body::empty());

        let res = token(&app.pool, &mut body, response).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().get("token").is_some());

        Ok(())
    }
}
