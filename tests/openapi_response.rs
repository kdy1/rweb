#![cfg(feature = "openapi")]

use rweb::*;
use serde::Serialize;

#[derive(Debug, Serialize, Schema)]
struct Resp<T> {
    status: usize,
    data: T,
}

#[get("/")]
fn index() -> Json<Resp<()>> {
    unimplemented!()
}

#[derive(Debug, Serialize, Schema)]
struct Product {}

#[get("/product")]
#[response(400)]
#[response(404)]
fn product() -> Json<Product> {
    unimplemented!()
}

#[get("/product")]
#[response(400, description = "Invalid query")]
#[response(400, description = "Invalid header")]
#[response(404, description = "No product matches the query")]
fn products() -> Json<Vec<Product>> {
    unimplemented!()
}

#[test]
fn component_test() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    panic!()
}