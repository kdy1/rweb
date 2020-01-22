use http::Method;
use rweb::*;

#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[tokio::main]
async fn main() {
    let (spec, filter) = openapi::spec(|| {
        // Build filters

        sum()
    });

    panic!("Spec: {:?}", spec);
    //    serve(filter);
}
