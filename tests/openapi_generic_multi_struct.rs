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
#[schema(component = "GenericStruct")]
struct GenericStruct<A, B> {
    a: A,
    b: B,
}

#[get("/")]
fn test_r(
    _: Query<GenericStruct<Option<One>, u64>>,
    _: Json<Vec<GenericStruct<Vec<Two>, Option<GenericStruct<One, One>>>>>,
) -> String {
    String::new()
}

#[test]
fn test_multi_generics_compile() {
    let (spec, _) = openapi::spec().build(|| test_r());
    let schemas = &spec.components.as_ref().unwrap().schemas;
    println!("{}", serde_yaml::to_string(&schemas).unwrap());
    for (name, _) in schemas {
        assert!(name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-'))
    }
    assert!(schemas.contains_key("One"));
    assert!(schemas.contains_key("Two"));
    assert!(schemas.contains_key("One_Opt"));
    assert!(schemas.contains_key("Two_List"));
    assert!(schemas.contains_key("GenericStruct-_One_One_-_Opt"));
}
