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
    pub paging_token: String,
}

#[get("/products")]
fn products(_: Query<SearchReq>) -> Vec<Product> {
    vec![]
}

#[get("/product/{id}")]
fn product(id: String) -> Product {
    Product {
        title: format!("Title of {}", id),
        id,
    }
}

#[test]
fn simple() {
    let (spec, _) = openapi::spec(|| {
        //
        product().or(products())
    });

    assert!(spec.paths.get("/products").is_some());
    assert!(spec.paths.get("/products").unwrap().get.is_some());

    assert!(spec.paths.get("/product/{id}").is_some());
    assert!(spec.paths.get("/product/{id}").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    panic!();
}

#[derive(Debug, Default, Serialize, Schema)]
struct Resp<T> {
    data: T,
}

#[derive(Debug, Default, Serialize, Schema)]
struct Data {}

#[get("/proxy")]
fn proxy() -> Json<Resp<Data>> {
    Resp {
        data: Data::default(),
    }
    .into()
}

#[test]
fn generic() {
    let (spec, _) = openapi::spec(|| {
        //
        proxy()
    });

    assert!(spec.paths.get("/proxy").is_some());
    assert!(spec.paths.get("/proxy").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    panic!();
}
