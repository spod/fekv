use hyper::{Body, Method, Request, Response, StatusCode};
use std::net::SocketAddr;

static INDEX: &[u8] =
    b"<html><head><title>kv</title></head><body><h1>kv</h1>A simple Key Value store! <br /><br /> \
Try POSTing data to <code>/kv/{key}</code> then GETing it back. <br /> <br />\
<pre> \
$ curl localhost:3000/kv/foo -XPOST -d 'bar' <br /> \
$ curl localhost:3000/kv/foo \
</pre></body></html>";

static TODO: &[u8] = b"TODO";

// base router which calls our other handlers or returns 404
pub async fn router(req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, hyper::Error> {
    println!("request: {} from client: {}", req.uri(), addr);

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(INDEX.into())),

        (&Method::GET, "/hello") => hello(req).await,

        // TODO need to match on "/kv/{foo}" not just "/kv" ...
        (&Method::GET, "/kv") | (&Method::POST, "/kv") => Ok(Response::new(TODO.into())),

        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub async fn hello(_: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello World!")))
}
