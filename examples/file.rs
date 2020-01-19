#![deny(warnings)]

use rweb::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let readme = rweb::get()
        .and(rweb::path::end())
        .and(rweb::fs::file("./README.md"));

    // dir already requires GET...
    let examples = rweb::path("ex").and(rweb::fs::dir("./examples/"));

    // GET / => README.md
    // GET /ex/... => ./examples/..
    let routes = readme.or(examples);

    rweb::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
