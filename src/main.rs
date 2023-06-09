//
// Server entrypoint - creates a store::MemStore instance (shared_store)
// and has lots of hyper.rs/tokio example copy/paste to set up web server
// The web service entrypoint is handlers::router(...)
//

use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::Mutex;

mod handlers;
mod kvstore;
pub mod raftstore;

use crate::handlers::router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // let shared_store = Arc::new(Mutex::new(store::memstore::MemStore::new()));
    let shared_store = Arc::new(Mutex::new(kvstore::diskstore::DiskKVStore::new()));

    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let addr = conn.remote_addr();
        let shared_store = shared_store.clone();
        let service = service_fn(move |req| router(req, addr, shared_store.to_owned()));
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
