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
        id,
        title: format!("Title of {}", id),
    }
}

#[test]
fn test1() {
    let (spec, _) = openapi::spec(|| {
        //
        product().or(product())
    });

    println!("{}", serde_yaml::to_string(&spec).unwrap());

    panic!();
}
