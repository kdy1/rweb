use rweb::*;

/// Adds a and b
#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[router("/math", services(sum))]
#[openapi(tags("math"))]
fn math() {}

#[tokio::main]
async fn main() {
    let (spec, filter) = openapi::spec(|| {
        // Build filters

        math().or(products::products())
    });

    panic!("Spec: {:?}", spec);
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
    fn list() -> Vec<Product> {}
}
