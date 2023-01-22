use std::{convert::Infallible, net::SocketAddr};

use apilib::App;
use dblib::connect;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Method, Server,
};
use tower_http::cors::{Any, Cors};
use towerlib::logging::Logging;

#[tokio::main]
async fn main() {
    env_logger::init();

    let pool = connect("users").await.unwrap();

    let app = App::new(pool, None);

    let make_service = make_service_fn(move |_: &AddrStream| {
        // Clone for each invocation of make_service
        let app = app.clone();

        let svc = service_fn(move |req| users::handle(app.clone(), req));
        let svc = Logging::new(svc);
        let svc = Cors::new(svc)
            .allow_methods([Method::GET, Method::POST])
            .allow_origin(Any);

        async move { Ok::<_, Infallible>(svc) }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
}
