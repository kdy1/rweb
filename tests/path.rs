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
fn index() -> Result<String, Error> {
    Err(Error {})
}

#[get("/foo")]
fn foo() -> Result<String, Error> {
    Ok(String::new())
}

#[get("/param/{foo}")]
fn param(foo: String) -> Result<String, Error> {
    Ok(foo)
}

#[get("/param/{v}")]
fn param_typed(v: u32) -> Result<String, Error> {
    Ok(v.to_string())
}

#[get("/param/{name}/{value}")]
fn multiple_param(name: String, value: String) -> Result<String, Error> {
    Ok(format!("{}={}", name, value))
}

#[get("/param/{name}/{value}")]
fn multiple_param_ordered(name: String, value: u8) -> Result<String, Error> {
    Ok(format!("{}={}", name, value))
}

#[get("/param/{name}/{value}")]
fn multiple_param_unordered(value: u8, name: String) -> Result<String, Error> {
    Ok(format!("{}={}", name, value))
}

#[test]
fn bind() {
    rweb::serve(
        index()
            .or(foo())
            .or(param())
            .or(param_typed())
            .or(multiple_param())
            .or(multiple_param_ordered())
            .or(multiple_param_unordered()),
    );
}
