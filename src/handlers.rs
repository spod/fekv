use hyper::{Body, Method, Request, Response, StatusCode, Uri};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use url::Url;

use crate::store::MemStore;

static INDEX: &[u8] =
    b"<html><head><title>kv</title></head><body><h1>kv</h1>A simple Key Value store! <br /><br /> \
Try POSTing data to <code>/kv/{key}</code> then GETing it back. <br /> <br />\
<pre> \
$ curl localhost:3000/kv/foo -XPOST -d 'bar' <br /> \
$ curl localhost:3000/kv/foo \
</pre></body></html>";

static TODO: &[u8] = b"TODO";

// route_root returns first segment of uri path
//   for example "/foo/bar" -> "/foo", "/" -> "/", "/blah/blah?blah" -> "/blah" etc
// also returns remainder of path and query parameters (if any)
// TODO tests and error handling (too many unwraps!)
fn route_root(req_uri: &Uri) -> (String, Option<String>, Option<String>) {
    // url crate only parses absolute urls so use example.com as fake base for url
    let root_url = Url::parse("http://example.com").unwrap();
    let request_url = root_url.join(req_uri.to_string().as_str()).unwrap();
    let path_segments: Vec<&str> = request_url.path_segments().unwrap().collect();
    let query: Option<String> = request_url.query().map(str::to_string);
    match path_segments.len() {
        0 => return (String::from("/"), None, query),
        1 => return (format!("/{}", path_segments[0]), None, query),
        _ => {
            let rest = path_segments.split_at(1).1;
            return (
                format!("/{}", path_segments[0]),
                Some(rest.join("/")),
                query,
            );
        }
    }
}

// base router which calls our other handlers or returns 404
pub async fn router(
    req: Request<Body>,
    addr: SocketAddr,
    _store: Arc<Mutex<MemStore<'_>>>,
) -> Result<Response<Body>, hyper::Error> {
    let (route, rest, _query) = route_root(req.uri());
    let route = route.as_str();
    let rest = rest.unwrap_or(String::from(""));

    if route != "/favicon.ico" {
        println!(
            "request: {} from client: {} with route: {}",
            req.uri(),
            addr,
            route
        );
    }

    // println!("store: {:?}", store);
    match (req.method(), route) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(INDEX.into())),

        (&Method::GET, "/hello") => hello(req, rest).await,

        (&Method::GET, "/kv") | (&Method::POST, "/kv") => Ok(Response::new(TODO.into())),

        (&Method::GET, "/favicon.ico") => response_404().await,
        _ => {
            println!("unknown route: {}, returning 404 ...", route);
            response_404().await
        }
    }
}

pub async fn hello(_req: Request<Body>, name: String) -> Result<Response<Body>, hyper::Error> {
    if name != "" {
        return Ok(Response::new(Body::from(format!("Hello {}!", name))));
    }
    Ok(Response::new(Body::from("Hello World!")))
}

pub async fn response_404() -> Result<Response<Body>, hyper::Error> {
    let mut not_found = Response::default();
    *not_found.status_mut() = StatusCode::NOT_FOUND;
    Ok(not_found)
}
