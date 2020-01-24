#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};
use serde_yaml;

#[get("/")]
fn index(_: Json<Enum>) -> String {
    String::new()
}

#[derive(Debug, Serialize, Deserialize, Schema)]
enum Enum {
    A(String),
    B(usize),
    Ref {
        /// Reference-foo-bar-baz
        ref_path: String,
    },
}

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("Reference-foo-bar-baz"));
}
