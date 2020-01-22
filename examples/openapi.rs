use rweb::*;

#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[router("/math", services(sum))]
fn math() {}

#[tokio::main]
async fn main() {
    let (spec, filter) = openapi::spec(|| {
        // Build filters

        sum().or(math())
    });

    panic!("Spec: {:?}", spec);
    //    serve(filter);
}
