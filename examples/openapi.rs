use rweb::*;
use rweb_openapi::{to_yaml, OpenApi};

#[tokio::main]
async fn main() {
    let (spec, filter) = openapi::spec(|| {
        // Build filters

        math::math().or(products::products())
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

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Product {
        pub id: String,
        pub title: String,
    }

    #[router("/products", services(list))]
    #[openapi(tags("products"))]
    pub fn products() {}

    #[get("/")]
    #[openapi(id = "products.list")]
    #[openapi(summary = "List products")]
    fn list() -> Json<Vec<Product>> {
        vec![].into()
    }
}
