#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Schema)]
struct RenameField {
    #[serde(rename = "result")]
    data: String,
}

#[get("/")]
fn component(_: Query<RenameField>) -> String {
    String::new()
}

#[test]
fn component_test() {
    let (spec, _) = openapi::spec().build(|| {
        //
        component()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("result"));
}

#[derive(Debug, Deserialize, Schema)]
struct ExampleReq {
    #[schema(example = "10")]
    limit: usize,
    data: String,
}
