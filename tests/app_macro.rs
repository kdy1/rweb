use rweb::{get, App};

struct Error {}

impl From<Error> for rweb::error::Error {
    fn from(_: Error) -> Self {
        unreachable!()
    }
}

#[get("/")]
pub fn index() -> Result<String, Error> {
    Err(Error {})
}

#[get("/foo")]
pub fn foo() -> Result<String, Error> {
    Ok(String::new())
}

#[test]
fn app_service() {}
