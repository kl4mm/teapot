use std::{convert::Infallible, net::SocketAddr};

use dblib::connect;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Server,
};
use towerlib::logging::Logging;
use users::App;

#[tokio::main]
async fn main() {
    env_logger::init();

    let pool = connect("users").await.unwrap();

    let app = App::new(pool);

    let make_service = make_service_fn(move |_: &AddrStream| {
        // Clone for each invocation of make_service
        let app = app.clone();

        let svc = service_fn(move |req| users::handle(app.clone(), req));
        let svc = Logging::new(svc);

        async move { Ok::<_, Infallible>(svc) }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
}
