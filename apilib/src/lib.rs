use hyper::{Body, Response, StatusCode};

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
