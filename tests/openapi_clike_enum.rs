#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[get("/")]
fn index(_: Json<Color>) -> String {
    String::new()
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[schema(component = "Color")]
pub enum Color {
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "blue")]
    Blue,
}

impl std::str::FromStr for Color {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "black" => Ok(Color::Black),
            "blue" => Ok(Color::Blue),
            _ => Err("ERR"),
        }
    }
}

#[test]
fn description() {
    let (spec, _) = openapi::spec().build(|| {
        //
        index()
    });
    let schemas = &spec.components.as_ref().unwrap().schemas;
    macro_rules! component {
        ($cn:expr) => {
            match schemas.get($cn) {
                Some(openapi::ObjectOrReference::Object(s)) => s,
                Some(..) => panic!("Component schema can't be a reference"),
                None => panic!("No component schema for {}", $cn),
            }
        };
    }
    let schema = component!("Color");
    assert_eq!(schema.schema_type, Some(openapi::Type::String));
    assert_eq!(schema.enum_values, vec!["black", "blue"]);
}
