#![cfg(feature = "openapi")]

use rweb::{
    rt::{Cow, IndexMap},
    *,
};
use rweb_openapi::v3_0::Response;
use serde::Serialize;

#[derive(Debug, Schema)]
enum Error {}

impl openapi::ResponseEntity for Error {
    fn describe_responses(_: &mut openapi::ComponentDescriptor) -> openapi::Responses {
        let mut map = IndexMap::new();

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
}

#[derive(Debug, Serialize, Schema)]
#[schema(component = "Item")]
struct Item {}

#[get("/item")]
fn item() -> Result<Json<Item>, Error> {
    unimplemented!()
}

#[test]
fn component_in_response() {
    let (spec, _) = openapi::spec().build(|| item());
    assert!(spec.paths.get("/item").is_some());
    assert!(spec.paths.get("/item").unwrap().get.is_some());
    assert!(spec.components.unwrap().schemas.get("Item").is_some());
}

#[get("/errable")]
#[openapi(response(code = "417", description = "🍵"))]
#[openapi(response(code = "5XX", description = "😵"))]
#[openapi(response(code = 200, description = "🐛"))]
#[openapi(response(code = 201, description = "✨", schema = "Json<Resp<String>>"))]
fn errable() -> Json<()> {
    unimplemented!()
}

#[test]
fn response_code_in_response() {
    let (spec, _) = openapi::spec().build(|| errable());
    let op = spec.paths.get("/errable").unwrap().get.as_ref().unwrap();
    assert!(op.responses.get("417").is_some());
    assert_eq!(op.responses.get("417").unwrap().description, "🍵");
    assert!(op.responses.get("5XX").is_some());
    assert_eq!(op.responses.get("5XX").unwrap().description, "😵");
    assert_eq!(op.responses.get("200").unwrap().description, "🐛");
    assert!(op
        .responses
        .get("201")
        .unwrap()
        .content
        .get("application/json")
        .is_some())
}
