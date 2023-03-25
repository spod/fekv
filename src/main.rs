use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::convert::Infallible;

mod handlers;
mod store;

use crate::handlers::router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let addr = conn.remote_addr();
        let service = service_fn(move |req| router(req, addr));
        async move { Ok::<_, Infallible>(service) }
    });

    let listen_addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&listen_addr).serve(make_svc);

    println!("Listening on http://{}", listen_addr);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}
