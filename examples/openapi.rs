use rweb::*;
use rweb_openapi::{to_yaml, OpenApi};

#[tokio::main]
async fn main() {
    let (spec, filter) = openapi::spec(|| {
        // Build filters

        math::math()
            .or(products::products())
            .or(generic::body())
            .or(generic::option())
    });

    println!("{}", to_yaml(&OpenApi::V3_0(spec)).unwrap());

    panic!();
}

mod math {
    use rweb::*;

    #[router("/math", services(sum))]
    #[openapi(tags("math"))]
    pub fn math() {}

    /// Adds a and b
    #[get("/sum/{a}/{b}")]
    fn sum(a: usize, b: usize) -> String {
        (a + b).to_string()
    }
}

mod products {
    use rweb::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct Product {
        pub id: String,
        pub title: String,
    }

    #[router("/products", services(list, product))]
    #[openapi(tags("products"))]
    pub fn products() {}

    #[get("/")]
    #[openapi(id = "products.list")]
    #[openapi(summary = "List products")]
    fn list() -> Json<Vec<Product>> {
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

mod generic {
    use rweb::{openapi::Entity, *};
    use rweb_openapi::v3_0::Schema;
    use serde::Deserialize;
    use std::collections::BTreeMap;

    #[derive(Debug, Deserialize)]
    struct LoginForm {
        id: String,
    }

    #[post("/login")]
    pub fn body(_: Json<LoginForm>) -> String {
        String::new()
    }

    #[post("/option")]
    pub fn option(_: Option<Json<LoginForm>>) -> String {
        String::new()
    }

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
}
