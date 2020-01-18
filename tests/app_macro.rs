use rweb::{get, App};

struct Error {}

impl From<Error> for rweb::error::Error {
    fn from(_: Error) -> Self {
        unreachable!()
    }
}

#[test]
fn app_service() {
    #[get("/")]
    fn index() -> Result<String, Error> {
        Err(Error {})
    }

    #[get("/foo")]
    fn foo() -> Result<String, Error> {
        Ok(String::new())
    }

    App::new().service(index).service(foo);
}
