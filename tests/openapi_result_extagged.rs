#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[derive(Schema, Serialize, Deserialize, Debug)]
#[schema(component = "IHazResult")]
pub struct IHazResult {
    result: Result<i64, String>,
}

#[get("/")]
fn index(_: Json<IHazResult>) -> String {
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
    macro_rules! component {
        ($cn:expr) => {
            match schemas.get($cn) {
                Some(openapi::ObjectOrReference::Object(s)) => s,
                Some(..) => panic!("Component schema can't be a reference"),
                None => panic!("No component schema for {}", $cn),
            }
        };
    }
    macro_rules! object {
        ($o:expr) => {
            match $o {
                openapi::ObjectOrReference::Object(s) => s,
                _ => panic!("Expected object, not reference"),
            }
        };
    }
    let res = &component!("IHazResult").properties["result"];
    assert_eq!(
        object!(&res.one_of[0]).schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(
        object!(&res.one_of[0]).properties["Ok"].schema_type,
        Some(openapi::Type::Integer)
    );
    assert_eq!(
        object!(&res.one_of[1]).schema_type,
        Some(openapi::Type::Object)
    );
    assert_eq!(
        object!(&res.one_of[1]).properties["Err"].schema_type,
        Some(openapi::Type::String)
    );
}
