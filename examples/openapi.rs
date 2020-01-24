use rweb::*;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    let (spec, filter) = openapi::spec().build(|| {
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
    use rweb::*;
    use serde::Serialize;

    #[router("/response", services(json))]
    pub fn response() {}

    #[derive(Debug, Serialize, Schema)]
    pub struct Data {
        msg: String,
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
    use rweb::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, Serialize, Deserialize, Schema)]
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

#[derive(Debug, Deserialize, Schema)]
struct SearchReq {
    query: String,
}

mod generic {
    use super::SearchReq;
    use rweb::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Schema)]
    struct LoginForm {
        id: String,
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
