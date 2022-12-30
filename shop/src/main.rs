use std::{convert::Infallible, net::SocketAddr};

use apilib::App;
use hyper::{
    server::conn::AddrStream,
    service::{make_service_fn, service_fn},
    Server,
};
use redis::Client as RedisClient;
use towerlib::logging::Logging;

#[tokio::main]
async fn main() {
    env_logger::init();

    let pool = dblib::connect("shop").await.unwrap();
    let redis = RedisClient::open("redis://:redis@127.0.0.1/").unwrap();

    let app = App::new(pool, Some(redis));

    let make_service = make_service_fn(move |_: &AddrStream| {
        // Clone for each invocation of make_service
        let app = app.clone();

        let svc = service_fn(move |req| shop::handle(app.clone(), req));
        let svc = Logging::new(svc);

        async move { Ok::<_, Infallible>(svc) }
    });

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let server = Server::bind(&addr).serve(make_service);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e)
    }
}
