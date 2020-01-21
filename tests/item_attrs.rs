use rweb::*;

#[get("/")]
#[header("X-AuthUser", "test-uid")]
fn header_guard() -> String {
    unreachable!()
}

#[get("/")]
#[body_size(max = "8192")]
fn body_size() -> String {
    unreachable!()
}

#[get("/")]
#[cors(origins("example.com"), max_age = 600)]
fn cors_1() -> String {
    unreachable!()
}

#[get("/")]
#[cors(origins("example.com"), methods(get), max_age = 600)]
fn cors_2() -> String {
    unreachable!()
}

#[get("/")]
#[cors(origins("*"), methods(get), max_age = 600)]
fn cors_3() -> String {
    unreachable!()
}

#[get("/")]
#[cors(
    origins("*"),
    methods(get, post, patch, delete),
    headers("accept"),
    max_age = 600
)]
fn cors_4() -> String {
    unreachable!()
}
