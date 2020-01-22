use rweb::{openapi::spec, *};

#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[tokio::main]
async fn main() {
    let spec = spec(&sum()).await;
    panic!("Spec: {:?}", spec);
}
