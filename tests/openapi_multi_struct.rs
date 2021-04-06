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

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Three")]
pub struct Three {
    two: Two,
    list_of_opt_of_one: Vec<Option<One>>,
}

#[get("/")]
fn index(_: Query<One>, _: Json<Three>) -> String {
    String::new()
}

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });
    let schemas = &spec.components.as_ref().unwrap().schemas;
    println!("{}", serde_yaml::to_string(&schemas).unwrap());
    assert!(schemas.contains_key("One"));
    assert!(schemas.contains_key("Two"));
    assert!(schemas.contains_key("Three"));
    assert!(schemas.contains_key("One_Opt"));
    assert!(schemas.contains_key("One_Opt_List"));
    assert!(!schemas.contains_key("One_List"));
    assert!(!schemas.contains_key("Two_List"));
    assert!(!schemas.contains_key("Three_List"));
    assert!(!schemas.contains_key("Two_Opt"));
    assert!(!schemas.contains_key("Three_Opt"));
}
