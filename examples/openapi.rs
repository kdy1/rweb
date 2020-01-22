use rweb::{openapi::spec, *};

#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[tokio::test]
async fn sum_spec() {
    let spec = spec(sum());
    panic!("Spec: {:?}", spec);
}
