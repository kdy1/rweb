#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};

#[get("/")]
fn index(_: Json<Colors>) -> String {
    String::new()
}

#[derive(Debug, Serialize, Deserialize, Schema)]
struct Colors {
    colordef: Color,
    coloradj: ColorAdjtagged,
    colorint: ColorInttagged,
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

#[derive(Debug, Serialize, Deserialize, Schema)]
#[serde(tag = "tag", rename_all = "lowercase")]
#[schema(component = "ColorInttagged")]
enum ColorInttagged {
    Black,
    Blue,
}

#[derive(Debug, Serialize, Deserialize, Schema)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "tag", content = "content")]
#[schema(component = "ColorAdjtagged")]
enum ColorAdjtagged {
    Black,
    Blue,
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
    let schema = component!("ColorInttagged");
    assert_eq!(schema.schema_type, Some(openapi::Type::Object));
    assert_eq!(schema.required, vec!["tag"]);
    let schema = schema.properties["tag"].unwrap().unwrap();
    assert_eq!(schema.schema_type, Some(openapi::Type::String));
    assert_eq!(schema.enum_values, vec!["black", "blue"]);
    let schema = component!("ColorAdjtagged");
    assert_eq!(schema.schema_type, Some(openapi::Type::Object));
    assert_eq!(schema.required, vec!["tag"]);
    let schema = schema.properties["tag"].unwrap().unwrap();
    assert_eq!(schema.schema_type, Some(openapi::Type::String));
    assert_eq!(schema.enum_values, vec!["black", "blue"]);
}
