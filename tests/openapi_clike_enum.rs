#![cfg(feature = "openapi")]

use rweb::*;
use serde::{Deserialize, Serialize};
use serde_yaml;

#[get("/")]
fn index(_: Json<Color>) -> String {
    String::new()
}

#[derive(Debug, Serialize, Deserialize, Schema)]
pub enum Color {
    Black,
    Blue,
}

impl std::str::FromStr for Color {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "black" => Ok(Color::BLACK),
            "blue" => Ok(Color::BLUE),
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

    assert!(spec.paths.get("/").is_some());
    assert!(spec.paths.get("/").unwrap().get.is_some());

    let yaml = serde_yaml::to_string(&spec).unwrap();
    println!("{}", yaml);

    assert!(yaml.contains("enum:"));
}
