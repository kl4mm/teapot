use std::{collections::HashMap, sync::Arc};

use hyper::{http::HeaderValue, Body, Response, StatusCode};
use redis::Client as RedisClient;
use serde_json::json;
use sqlx::{Pool, Postgres};

pub struct App {
    pub pool: Pool<Postgres>,
    pub redis: Option<RedisClient>,
}

impl App {
    pub fn new(pool: Pool<Postgres>, redis: Option<RedisClient>) -> Arc<Self> {
        Arc::new(Self { pool, redis })
    }
}

const INTERNAL_SERVER_ERROR: &str = r#"{"message": "Internal Server Error"}"#;
const UNPROCESSABLE_ENTITY: &str = r#"{"message": "Unprocessable Entity"}"#;

pub fn set_response(
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

pub fn set_response_v2(
    mut response: Response<Body>,
    err: (StatusCode, Option<serde_json::Value>),
) -> Response<Body> {
    *response.status_mut() = err.0;
    response
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));

    let body = match err.1 {
        Some(m) => Body::from(m.to_string()),
        None => Body::from(json!({ "message": err.0.to_string() }).to_string()),
    };

    *response.body_mut() = body;

    response
}

pub fn parse_query(query: Option<&str>) -> HashMap<&str, &str> {
    let mut query_map = HashMap::new();
    if query.is_none() {
        return query_map;
    }

    let queries: Vec<&str> = query.unwrap().split("&").collect();

    for query in queries {
        let (k, v) = match query.split_once("=") {
            Some(kv) => kv,
            None => continue,
        };

        query_map.insert(k, v);
    }

    query_map
}
