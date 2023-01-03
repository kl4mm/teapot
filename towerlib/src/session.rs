use hyper::{http::HeaderValue, HeaderMap, Request, StatusCode};
use rand::Rng;
use tower::Service;

pub const SESSION_ID: &str = "Session-ID";

#[derive(Clone)]
pub struct Session<S> {
    inner: S,
}

impl<S> Session<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S, B> Service<Request<B>> for Session<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        let session_id = req.headers().get(SESSION_ID);

        if let Some(id) = session_id {
            // TODO: verify ID, set future type, if invalid return error
            log::info!("{:?}", id);
        } else {
            let session_id = HeaderValue::from_str(&gen_session()).unwrap();
            req.headers_mut().insert(SESSION_ID, session_id);
        };

        self.inner.call(req)
    }
}

pub fn gen_session() -> String {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..100_000).to_string()
}

pub fn get_session(headers: &HeaderMap) -> Result<&str, StatusCode> {
    let session = match headers.get(SESSION_ID) {
        Some(s) => s.to_str().map_err(|e| {
            log::error!("{}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
        // Theres was a problem assigning a session ID
        // in the middleware:
        None => Err(StatusCode::INTERNAL_SERVER_ERROR)?,
    };

    Ok(session)
}
