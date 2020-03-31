use rweb::*;
use serde::Serialize;
#[derive(Clone, Serialize)]
struct User {
    id: u32,
    name: String,
}
#[get("/hi")]
fn hi() -> &'static str {
    "Hello, World!"
}

#[get("/ping/{word}")]
async fn ping(word: String) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    // a cast is needed for now, see https://github.com/rust-lang/rust/issues/60424
    Ok(Box::new(format!("pong {}", word)) as Box<dyn warp::Reply>)
}
#[get("/user")]
fn display_user(#[data] user: User) -> impl Reply {
    rweb::reply::json(&user)
}

#[tokio::main]
async fn main() {
    let user = User {
        id: 1,
        name: "Christoffer".to_string(),
    };
    // Sending user so it can be used in display user. Usually you would send a
    // db_connection or something.
    let routes = routes![hi, ping].or(routes![user; display_user]);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
