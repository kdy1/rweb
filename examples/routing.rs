#![deny(warnings)]

use rweb::*;

#[get("/hi")]
fn hi() -> &'static str {
    "Hello, World!"
}

/// How about multiple segments? First, we could use the `path!` macro:
#[get("/hello/from/warp")]
fn hello_from_warp() -> &'static str {
    "Hello from warp!"
}

/// Fine, but how do I handle parameters in paths?
#[get("/sum/{a}/{b}")]
fn sum(a: u32, b: u32) -> String {
    format!("{} + {} = {}", a, b, a + b)
}

/// Any type that implements FromStr can be used, and in any order:
#[get("/{a}/times/{b}")]
fn times(a: u16, b: u16) -> String {
    format!("{} times {} = {}", a, b, a * b)
}

/// What! And? What's that do?
///
/// It combines the filters in a sort of "this and then that" order. In
/// fact, it's exactly what the `path!` macro has been doing internally.
#[get("/byt/{name}")]
fn bye(name: String) -> String {
    format!("Good bye, {}!", name)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // We'll start simple, and gradually show how you combine these powers
    // into super powers!

    // Oh shoot, those math routes should be mounted at a different path,
    // is that possible? Yep.
    //
    // GET /math/sum/:u32/:u32
    // GET /math/:u16/times/:u16
    let math = rweb::path("math");
    let _sum = math.and(sum());
    let _times = math.and(times());

    // Ah, can filters do things besides `and`?
    //
    // Why, yes they can! They can also `or`! As you might expect, `or` creates
    // a "this or else that" chain of filters. If the first doesn't succeed,
    // then it tries the other.
    //
    // So, those `math` routes could have been mounted all as one, with `or`.
    //
    // GET /math/sum/:u32/:u32
    // GET /math/:u16/times/:u16
    let math = rweb::path("math").and(sum().or(times()));

    // We can use the end() filter to match a shorter path
    let help = rweb::path("math")
        // Careful! Omitting the following line would make this filter match
        // requests to /math/sum/:u32/:u32 and /math/:u16/times/:u16
        .and(rweb::path::end())
        .map(|| "This is the Math API. Try calling /math/sum/:u32/:u32 or /math/:u16/times/:u16");
    let math = help.or(math);

    // Let's let people know that the `sum` and `times` routes are under `math`.
    let sum =
        sum().map(|output| format!("(This route has moved to /math/sum/:u16/:u16) {}", output));
    let times =
        times().map(|output| format!("(This route has moved to /math/:u16/times/:u16) {}", output));

    // It turns out, using `or` is how you combine everything together into
    // a single API. (We also actually haven't been enforcing the that the
    // method is GET, so we'll do that too!)
    //
    // GET /hi
    // GET /hello/from/warp
    // GET /bye/:string
    // GET /math/sum/:u32/:u32
    // GET /math/:u16/times/:u16

    let routes = rweb::get().and(
        hi().or(hello_from_warp())
            .or(bye())
            .or(math)
            .or(sum)
            .or(times),
    );

    // Note that composing filters for many routes may increase compile times
    // (because it uses a lot of generics). If you wish to use dynamic dispatch
    // instead and speed up compile times while making it slightly slower at
    // runtime, you can use Filter::boxed().

    rweb::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
