#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};
use serde_yaml;
use rweb_openapi::v3_0::{ObjectOrReference};

#[derive(Debug, Default, Serialize, Deserialize, Schema)]
pub struct Product {
    pub id: String,
    pub title: String,
}

#[post("/json")]
fn json(_: Json<Product>) -> String {
    String::new()
}

#[post("/form")]
fn form(_: Form<Product>) -> String { String::new() }

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        json().or(form())
    });

    assert!(spec.paths.get("/json").is_some());
    assert!(spec.paths.get("/form").is_some());

    assert!(spec.paths.get("/json").unwrap().post.is_some());
    assert!(spec.paths.get("/form").unwrap().post.is_some());

    assert!(spec.paths.get("/json").unwrap().post.as_ref().unwrap().request_body.is_some());
    assert!(spec.paths.get("/form").unwrap().post.as_ref().unwrap().request_body.is_some());

    match spec.paths.get("/json").unwrap().post.as_ref().unwrap().request_body.as_ref()
        .unwrap() {
        ObjectOrReference::Object(request_body) => {
            assert!(request_body.content.contains_key("application/json"));
        },
        ObjectOrReference::Ref {  .. } =>
            panic!("Struct Product dont have `#[schema(component = \"...\")]`"),
    }

    match spec.paths.get("/form").unwrap().post.as_ref().unwrap().request_body.as_ref()
        .unwrap() {
        ObjectOrReference::Object(request_body) => {
            assert!(request_body.content.contains_key("x-www-form-urlencoded"));
        },
        ObjectOrReference::Ref {  .. } =>
            panic!("Struct Product dont have `#[schema(component = \"...\")]`"),
    }

    let yaml = serde_yaml::to_string(&spec).unwrap();

    println!("{}", yaml);
}
