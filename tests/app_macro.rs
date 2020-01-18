use rweb::{get, App};

struct Error {}

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
