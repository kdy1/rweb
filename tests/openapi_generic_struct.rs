//! https://github.com/kdy1/rweb/issues/38

#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "GenericStruct")]
struct GenericStruct<A, B> {
    a: A,
    b: B,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "GenericStructWithConst")]
struct GenericStructWithConst<A, const D: usize> {
    a: A,
}

#[get("/")]
fn test_r(
    _: Query<GenericStruct<String, u64>>,
    _: Json<
        GenericStruct<
            GenericStruct<String, Vec<u64>>,
            GenericStruct<String, GenericStructWithConst<Vec<Vec<i32>>, 16>>,
        >,
    >,
) -> String {
    String::new()
}

#[test]
fn test_generics_compile() {
    let (spec, _) = openapi::spec().build(|| test_r());
    let schemas = &spec.components.as_ref().unwrap().schemas;
    println!("{}", serde_yaml::to_string(&schemas).unwrap());
    for (name, _) in schemas {
        assert!(name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '-'))
    }
}
