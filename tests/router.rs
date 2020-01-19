use rweb::*;

#[get("/sum/{a}/{b}")]
fn sum(a: usize, b: usize) -> String {
    (a + b).to_string()
}

#[get("/mul/{a}/{b}")]
fn mul(a: usize, b: usize) -> String {
    (a * b).to_string()
}

#[router("/math", services(sum, mul))]
struct MathRouter;

#[test]
fn router() {
    serve(MathRouter);
}
