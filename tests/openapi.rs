#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};
use serde_yaml;

#[derive(Debug, Default, Serialize, Deserialize, Schema)]
pub struct Product {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Schema)]
pub struct SearchReq {
    pub query: String,
    pub limit: usize,
    /// paging-token-example
    pub paging_token: String,
}

#[get("/products")]
fn products(_: Query<SearchReq>) -> Json<Vec<Product>> {
    vec![].into()
}

#[get("/product/{id}")]
fn product(id: String) -> Json<Product> {
    Product {
        title: format!("Title of {}", id),
        id,
    }
    .into()
}

#[test]
fn simple() {
    let (spec, _) = openapi::spec().build(|| {
        //
        product().or(products())
    });

    assert!(spec.paths.get("/products").is_some());
    assert!(spec.paths.get("/products").unwrap().get.is_some());

    assert!(spec.paths.get("/product/{id}").is_some());
    assert!(spec.paths.get("/product/{id}").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("paging-token-example"));
}

#[derive(Debug, Default, Serialize, Schema)]
struct Resp<T> {
    /// http-status-code
    status: usize,
    data: T,
}

#[derive(Debug, Default, Serialize, Schema)]
struct Data {}

#[get("/proxy")]
fn proxy() -> Json<Resp<Data>> {
    Resp {
        status: 200,
        data: Data::default(),
    }
    .into()
}

// TODO: enum

#[test]
fn generic() {
    let (spec, _) = openapi::spec().build(|| {
        //
        proxy()
    });

    assert!(spec.paths.get("/proxy").is_some());
    assert!(spec.paths.get("/proxy").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("http-status-code"));
}

/// Doc comment
#[get("/")]
#[openapi(description = "foo-bar")]
/// Doc comment
fn index() -> String {
    String::new()
}

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());
    assert_eq!(
        spec.paths
            .get("/")
            .unwrap()
            .get
            .as_ref()
            .unwrap()
            .description,
        "foo-bar"
    );

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);
}
