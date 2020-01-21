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
