use hyper::{Body, Method, Request, Response, StatusCode, Uri};
use std::net::SocketAddr;
use url::Url;

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
// TODO tests and error handling (too many unwraps!)
fn route_root(req_uri: &Uri) -> String {
    // url crate only parses absolute urls so use example.com as fake base for url
    let root_url = Url::parse("http://example.com").unwrap();
    let request_url = root_url.join(req_uri.to_string().as_str()).unwrap();
    let path_segments: Vec<&str> = request_url.path_segments().unwrap().collect();
    format!("/{}", path_segments[0])
}

// base router which calls our other handlers or returns 404
pub async fn router(req: Request<Body>, addr: SocketAddr) -> Result<Response<Body>, hyper::Error> {
    let route = route_root(req.uri());
    let route = route.as_str();
    println!(
        "request: {} from client: {} - route: {}",
        req.uri(),
        addr,
        route
    );

    match (req.method(), route) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(INDEX.into())),

        (&Method::GET, "/hello") => hello(req).await,

        (&Method::GET, "/kv") | (&Method::POST, "/kv") => Ok(Response::new(TODO.into())),

        _ => {
            println!("unknown route: {}, returning 404 ...", route);
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

pub async fn hello(_: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello World!")))
}
