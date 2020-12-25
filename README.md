# rweb

[![Build Status](https://travis-ci.com/kdy1/rweb.svg?branch=master)](https://travis-ci.com/kdy1/rweb)

Yet another web server framework for rust.

Installation (without automatic openapi generation):

```toml
[dependencies]
rweb = "0.5"
tokio = "0.2"
```

# Features

- Safe & Correct

Since `rweb` is based on [warp][], which features safety and correctness, `rweb` has same property.

- Easy to read code

```rust
use rweb::*;
use serde::{Serialize, Deserialize};

#[get("/output")]
fn output() -> String {
    String::from("this returns 200 with text/plain mime type")
}

#[derive(Debug, Serialize, Deserialize, Schema)]
struct Product {
    id: String,
    title: String,
}

#[get("/products")]
fn products() -> Json<Vec<Product>> {
    // ...
    // This returns 200 with application/json
}

#[get("/products/{id}")]
fn product(id: String) -> Json<Product> {
    // ...
    // This returns 200 with application/json
}

#[get("/product")]
fn new_product(_product: Json<Product>) -> Json<Product> {
    // ...
    // This returns 200 with application/json
}

#[derive(Debug, Serialize, Deserialize, Schema)]
struct SearchOption {
    query: String,
    limit: usize,
    page_token: String,
}

#[get("/search")]
fn search(_product: Query<SearchOption>) -> Json<Vec<Product>> {
    // ...
    // This returns 200 with application/json
}

#[tokio::main]
async fn main() {
    serve(output().or(product()).or(products()).or(search())).run(([127, 0, 0, 1], 3030)).await;
}

```

- Websocket

If you want to use websocket, just declare a parameter typed `Ws`. It's all.

```rust
use rweb::*;

#[get("/ws")]
fn example(ws: ws::Ws) -> String {
    String::new("use ws.on_upgrade or extra")
}
```

- Automatic openapi spec generation

rweb supports automatically generating openapi specification file based on your code.

See: [documentation](https://docs.rs/rweb/latest/rweb/openapi/index.html) for usage.

# Comparison

| Name             | rweb                                                                                 | actix-web                                                                                  | gotham                                                                                      | iron                                                                                 | nickel                                                                                          | rocket                                                                                          | rouille                                                                                   | Thruster                                                                                  | Tide                                                                                    | tower-web                                                                                       | warp                                                                                        |
| ---------------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| License          | ![license](https://img.shields.io/crates/l/rweb.svg?label=%20)                       | ![license](https://img.shields.io/crates/l/actix-web.svg?label=%20)                        | ![license](https://img.shields.io/crates/l/gotham.svg?label=%20)                            | ![license](https://img.shields.io/crates/l/iron.svg?label=%20)                       | ![license](https://img.shields.io/crates/l/nickel.svg?label=%20)                                | ![license](https://img.shields.io/crates/l/rocket.svg?label=%20)                                | ![license](https://img.shields.io/crates/l/rouille.svg?label=%20)                         | ![license](https://img.shields.io/crates/l/Thruster.svg?label=%20)                        | ![license](https://img.shields.io/crates/l/tide.svg?label=%20)                          | ![license](https://img.shields.io/crates/l/tower-web.svg?label=%20)                             | ![license](https://img.shields.io/crates/l/warp.svg?label=%20)                              |
| Version          | ![version](https://img.shields.io/crates/v/rweb.svg?label=%20)                       | ![version](https://img.shields.io/crates/v/actix-web.svg?label=%20)                        | ![version](https://img.shields.io/crates/v/gotham.svg?label=%20)                            | ![version](https://img.shields.io/crates/v/iron.svg?label=%20)                       | ![version](https://img.shields.io/crates/v/nickel.svg?label=%20)                                | ![version](https://img.shields.io/crates/v/rocket.svg?label=%20)                                | ![version](https://img.shields.io/crates/v/rouille.svg?label=%20)                         | ![version](https://img.shields.io/crates/v/Thruster.svg?label=%20)                        | ![version](https://img.shields.io/crates/v/tide.svg?label=%20)                          | ![version](https://img.shields.io/crates/v/tower-web.svg?label=%20)                             | ![version](https://img.shields.io/crates/v/warp.svg?label=%20)                              |
| Recent downloads | ![recent downloads](https://img.shields.io/crates/dr/rweb.svg?label=%20)             | ![recent downloads](https://img.shields.io/crates/dr/actix-web.svg?label=%20)              | ![recent downloads](https://img.shields.io/crates/dr/gotham.svg?label=%20)                  | ![recent downloads](https://img.shields.io/crates/dr/iron.svg?label=%20)             | ![recent downloads](https://img.shields.io/crates/dr/nickel.svg?label=%20)                      | ![recent downloads](https://img.shields.io/crates/dr/rocket.svg?label=%20)                      | ![recent downloads](https://img.shields.io/crates/dr/rouille.svg?label=%20)               | ![recent downloads](https://img.shields.io/crates/dr/Thruster.svg?label=%20)              | ![recent downloads](https://img.shields.io/crates/dr/tide.svg?label=%20)                | ![recent downloads](https://img.shields.io/crates/dr/tower-web.svg?label=%20)                   | ![recent downloads](https://img.shields.io/crates/dr/warp.svg?label=%20)                    |
| Github stars     | ![github stars](https://img.shields.io/github/stars/kdy1/rweb.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/actix/actix-web.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/gotham-rs/gotham.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/iron/iron.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/nickel-org/nickel.rs.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/SergioBenitez/Rocket.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/tomaka/rouille.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/trezm/Thruster.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/http-rs/tide.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/carllerche/tower-web.svg?label=%20)         | ![github stars](https://img.shields.io/github/stars/seanmonstar/warp.svg?label=%20)         |
| Contributors     | ![contributors](https://img.shields.io/github/contributors/kdy1/rweb.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/actix/actix-web.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/gotham-rs/gotham.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/iron/iron.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/nickel-org/nickel.rs.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/SergioBenitez/Rocket.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/tomaka/rouille.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/trezm/Thruster.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/http-rs/tide.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/carllerche/tower-web.svg?label=%20)  | ![contributors](https://img.shields.io/github/contributors/seanmonstar/warp.svg?label=%20)  |
| Activity         | ![activity](https://img.shields.io/github/commit-activity/m/kdy1/rweb.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/actix/actix-web.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/gotham-rs/gotham.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/iron/iron.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/nickel-org/nickel.rs.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/SergioBenitez/Rocket.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/tomaka/rouille.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/trezm/Thruster.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/http-rs/tide.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/carllerche/tower-web.svg?label=%20) | ![activity](https://img.shields.io/github/commit-activity/m/seanmonstar/warp.svg?label=%20) |
| Base framework   | hyper / warp                                                                         | tokio                                                                                      | hyper                                                                                       | hyper                                                                                | hyper                                                                                           | hyper                                                                                           | tiny-http                                                                                 | tokio                                                                                     | hyper                                                                                   | hyper                                                                                           | hyper                                                                                       |
| https            | Y                                                                                    | Y                                                                                          | Y                                                                                           | ?                                                                                    | ?                                                                                               | ?                                                                                               | ?                                                                                         | ?                                                                                         | ?                                                                                       | ?                                                                                               | Y                                                                                           |
| http 2           | Y                                                                                    | Y                                                                                          | ?                                                                                           | ?                                                                                    | ?                                                                                               | ?                                                                                               | ?                                                                                         | ?                                                                                         | ?                                                                                       | ?                                                                                               | Y                                                                                           |
| async            | Y                                                                                    | Y                                                                                          | Y                                                                                           |                                                                                      |                                                                                                 |                                                                                                 |                                                                                           | Y                                                                                         | Y                                                                                       | Y                                                                                               | Y (via different method)                                                                    |
| stable rust      | Y                                                                                    | Y                                                                                          | Y                                                                                           | Y                                                                                    | Y                                                                                               |                                                                                                 | Y                                                                                         | Y                                                                                         | Y                                                                                       | Y                                                                                               | Y                                                                                           |
| openapi support  | Y                                                                                    |                                                                                            |                                                                                             |                                                                                      |                                                                                                 |                                                                                                 |                                                                                           |                                                                                           |                                                                                         |                                                                                                 |                                                                                             |

[warp]: https://github.com/seanmonstar/warp
