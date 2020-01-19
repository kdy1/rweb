use http::{Response, StatusCode};
use hyper::Body;
use rweb::{get, reply::Reply, Filter};

struct Error {}
impl Reply for Error {
    fn into_response(self) -> Response<Body> {
        StatusCode::from_u16(500).unwrap().into_response()
    }
}

#[get("/")]
pub fn index() -> Result<String, Error> {
    Err(Error {})
}

#[get("/foo")]
pub fn foo() -> Result<String, Error> {
    Ok(String::new())
}

#[get("/param/{foo}")]
pub fn param(foo: String) -> Result<String, Error> {
    Ok(String::new())
}

#[get("/param/{v}")]
pub fn param_typed(v: u32) -> Result<String, Error> {
    Ok(String::new())
}

#[test]
fn app_service() {
    rweb::serve(foo().or(index()));
}
