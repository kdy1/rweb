use http::{Response, StatusCode};
use hyper::Body;
use rweb::{get, reply::Reply, Filter};
use serde::Deserialize;

struct Error {}
impl Reply for Error {
    fn into_response(self) -> Response<Body> {
        StatusCode::from_u16(500).unwrap().into_response()
    }
}

#[derive(Deserialize)]
struct Req {}

#[get("/")]
fn index(body: rweb::Json<Req>) -> Result<String, Error> {
    Err(Error {})
}

#[test]
fn bind() {
    rweb::serve(index());
}
