use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use hyper::{Method, Request};
use pin_project::pin_project;
use tower::Service;

#[derive(Clone)]
pub struct Logging<S> {
    inner: S,
}

impl<S> Logging<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

// Futures wrapped in async aren't Unpin, this crate allows us to poll
// those futures:
#[pin_project]
pub struct LoggingFuture<F> {
    #[pin]
    future: F,
    method: Method,
    path: String,
    instant: Instant,
}

impl<F> Future for LoggingFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = match this.future.poll(cx) {
            Poll::Ready(res) => res,
            Poll::Pending => return Poll::Pending,
        };

        let duration = this.instant.elapsed();
        log::info!(
            "Finished processing {} {} in {:?}",
            this.method,
            this.path,
            duration
        );

        Poll::Ready(res)
    }
}

impl<S, B> Service<Request<B>> for Logging<S>
where
    S: Service<Request<B>> + Clone + Send + 'static,
    S::Future: Send,
    B: Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LoggingFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().path().to_owned();

        log::info!("Received a {} for {}", method, path);

        let instant = Instant::now();
        let future = LoggingFuture {
            future: self.inner.call(req),
            method,
            path,
            instant,
        };

        future
    }
}
