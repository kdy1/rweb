//! https://github.com/kdy1/rweb/issues/38

#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "One")]
pub struct One {}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Two")]
pub struct Two {}

#[get("/")]
fn index(_: Query<One>, _: Json<Two>) -> String {
    String::new()
}

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);
}
