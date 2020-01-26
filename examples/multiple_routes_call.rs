use rweb::*;
#[get("/hi")]
fn hi() -> &'static str {
    "Hello, World!"
}

#[get("/ping/{word}")]
async fn ping(word: String) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // a cast is needed for now, see https://github.com/rust-lang/rust/issues/60424
    Ok(Box::new(format!("pong {}", word)) as Box<dyn warp::Reply>)
}
#[tokio::main]
async fn main() {
    let routes = routes![hi, ping];
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
