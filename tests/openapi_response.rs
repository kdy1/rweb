#![cfg(feature = "openapi")]

use rweb::{
    rt::{BTreeMap, Cow},
    *,
};
use rweb_openapi::v3_0::Response;
use serde::Serialize;

#[derive(Debug, Schema)]
enum Error {}

impl openapi::ResponseEntity for Error {
    fn describe_responses() -> openapi::Responses {
        let mut map = BTreeMap::new();

        map.insert(
            Cow::Borrowed("404"),
            Response {
                description: Cow::Borrowed("Product not found"),
                ..Default::default()
            },
        );

        map
    }
}

#[derive(Debug, Serialize, Schema)]
struct Resp<T> {
    status: usize,
    data: T,
}

#[get("/")]
fn index() -> Result<Json<Resp<()>>, Error> {
    unimplemented!()
}

#[derive(Debug, Serialize, Schema)]
struct Product {}

#[get("/product")]
fn product() -> Result<Json<Product>, Error> {
    unimplemented!()
}

#[get("/product")]
fn products() -> Result<Json<Vec<Product>>, Error> {
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
