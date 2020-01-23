use rweb::{
    openapi::{Entity, Schema},
    *,
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() {
    let (spec, filter) = openapi::spec(|| {
        // Build filters

        math::math()
            .or(products::products())
            .or(generic::body())
            .or(generic::optional())
            .or(generic::search())
            .or(response::response())
    });

    println!("{}", serde_yaml::to_string(&spec).unwrap());

    panic!();
}

mod response {
    use rweb::{
        openapi::{Entity, Schema},
        *,
    };
    use serde::Serialize;
    use std::collections::BTreeMap;

    #[router("/response", services(json))]
    pub fn response() {}

    #[derive(Debug, Serialize)]
    pub struct Data {
        msg: String,
    }

    /// TODO: Replace this with derive
    impl Entity for Data {
        fn describe() -> Schema {
            let mut map = BTreeMap::new();

            map.insert("msg".into(), String::describe());

            Schema {
                schema_type: "object".into(),
                properties: map,
                ..Default::default()
            }
        }
    }

    #[get("/json")]
    pub fn json() -> Json<Data> {
        Json::from(Data {
            msg: "Hello".into(),
        })
    }
}

mod math {
    use rweb::*;

    #[router("/math", services(sum))]
    #[openapi(tags("math"))]
    pub fn math() {}

    /// Adds a and b
    /// and
    /// return it
    #[get("/sum/{a}/{b}")]
    fn sum(a: usize, b: usize) -> String {
        (a + b).to_string()
    }
}

mod products {
    use super::SearchReq;
    use rweb::{
        openapi::{Entity, Schema},
        *,
    };
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Product {
        pub id: String,
        pub title: String,
    }

    /// TODO: Replace this with derive
    impl Entity for Product {
        fn describe() -> Schema {
            let mut map = BTreeMap::new();

            map.insert("id".into(), String::describe());
            map.insert("title".into(), String::describe());

            Schema {
                schema_type: "object".into(),
                properties: map,
                ..Default::default()
            }
        }
    }

    #[router("/products", services(list, product))]
    #[openapi(tags("products"))]
    pub fn products() {}

    #[get("/")]
    #[openapi(id = "products.list")]
    #[openapi(summary = "List products")]
    fn list(_query: Query<SearchReq>) -> Json<Vec<Product>> {
        // Mix
        vec![].into()
    }

    #[get("/{id}")]
    #[openapi(id = "products.get")]
    #[openapi(summary = "Get a product")]
    fn product(id: String) -> Json<Product> {
        Product {
            id,
            title: Default::default(),
        }
        .into()
    }
}

#[derive(Debug, Deserialize)]
struct SearchReq {
    query: String,
}

/// TODO: Replace this with derive
impl Entity for SearchReq {
    fn describe() -> Schema {
        let mut map = BTreeMap::new();

        map.insert("query".into(), String::describe());

        Schema {
            schema_type: "object".into(),
            properties: map,
            ..Default::default()
        }
    }
}

mod generic {
    use super::SearchReq;
    use rweb::{openapi::Entity, *};
    use rweb_openapi::v3_0::Schema;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Debug, Deserialize)]
    struct LoginForm {
        id: String,
    }

    /// TODO: Replace this with derive
    impl Entity for LoginForm {
        fn describe() -> Schema {
            let mut map = BTreeMap::new();

            map.insert("id".into(), String::describe());

            Schema {
                schema_type: "object".into(),
                properties: map,
                ..Default::default()
            }
        }
    }

    #[post("/login")]
    #[openapi(tags("auth"))]
    pub fn body(_: Json<LoginForm>) -> String {
        String::new()
    }

    #[post("/optional")]
    pub fn optional(_: Option<Json<LoginForm>>) -> String {
        String::new()
    }

    #[post("/search")]
    pub fn search(_: Option<Query<SearchReq>>) -> String {
        String::new()
    }
}
