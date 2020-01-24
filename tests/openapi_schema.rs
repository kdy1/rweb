#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Item")]
struct ComponentTestReq {
    data: String,
}

#[get("/")]
fn component(_: Query<ComponentTestReq>) -> String {
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

    panic!()
}
