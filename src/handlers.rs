//
// All http server handling
//
// Key functions are:
//   - route_root(...) - helper to return the "route" from a uri
//   - router(...) - http entrypoint which routes to other handlers as appropriate
//   - hello(...) - hello world!
//   - kv(...) - REST interface to store backend
//

use hyper::{Body, Method, Request, Response, StatusCode, Uri};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

use crate::store::Storage;

static INDEX: &[u8] =
    b"<html><head><title>fekv</title></head><body><h1>fekv</h1>A Toy Key Value store! <br /><br /> \
Try PUT/POSTing data to <code>/kv/{key}</code> then GETing it back. <br /> <br />\
<pre> \
$ curl --fail -X 'PUT' localhost:3000/fekv/foo -d 'bar' <br /> \
$ curl --fail -X 'GET' localhost:3000/fekv/foo <br /> \
$ curl --fail -X 'DELETE' localhost:3000/kv/foo <br /> \
$ curl --fail -X 'GET' localhost:3000/fekv/foo \
</pre></body></html>";

static OK: &[u8] = b"OK";

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
    store: Arc<Mutex<impl Storage>>,
) -> Result<Response<Body>, hyper::Error> {
    let (route, rest, _query) = route_root(req.uri());
    let route = route.as_str();
    let rest = rest.unwrap_or(String::from(""));

    if route != "/favicon.ico" {
        println!(
            "request: {} {} from client: {} with route: {}",
            req.method(),
            req.uri(),
            addr,
            route
        );
    }

    match (req.method(), route) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(INDEX.into())),

        (&Method::GET, "/hello") => hello(req, rest).await,

        (&Method::DELETE, "/fekv")
        | (&Method::GET, "/fekv")
        | (&Method::POST, "/fekv")
        | (&Method::PUT, "/fekv") => fekv_handler(req, rest, store).await,

        (&Method::GET, "/favicon.ico") => response_404().await,
        _ => {
            println!("unknown route: {}, returning 404 ...", route);
            response_404().await
        }
    }
}

pub async fn fekv_handler(
    req: Request<Body>,
    key: String,
    store: Arc<Mutex<impl Storage>>,
) -> Result<Response<Body>, hyper::Error> {
    match req.method() {
        &Method::GET => {
            let st = store.lock().await;
            let val = st.get(key.to_string());
            match val {
                Ok(val) => return Ok(Response::new(val.into())),
                Err(_err) => return response_404().await,
            }
        }
        &Method::POST | &Method::PUT => {
            let b = hyper::body::to_bytes(req).await.unwrap();
            let mut st = store.try_lock_owned().unwrap();
            let res = st.set(key.to_string(), b.to_vec());
            match res {
                Ok(_res) => return Ok(Response::new(OK.into())),
                Err(_err) => return response_404().await,
            }
        }
        &Method::DELETE => {
            let mut st = store.try_lock_owned().unwrap();
            let res = st.delete(key.to_string());
            match res {
                Ok(_res) => return Ok(Response::new(OK.into())),
                Err(_err) => return response_404().await,
            }
        }
        _ => {
            println!("invalid request method: {} returning 404 ...", req.method());
            return response_404().await;
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
