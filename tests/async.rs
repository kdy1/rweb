use http::Error;
use rweb::{get, serve, Filter};

async fn task() -> Result<String, Error> {
    Ok(String::new())
}

#[get("/")]
async fn index() -> Result<String, Error> {
    task().await
}

#[get("/foo")]
async fn foo() -> Result<String, Error> {
    task().await
}

#[get("/param/{foo}")]
async fn param(foo: String) -> Result<String, Error> {
    println!("{}", foo); // to use it
    task().await
}

#[test]
fn bind() {
    serve(index().or(foo()).or(param()));
}
