#[test]
fn app_service() {
    #[get("/")]
    fn index() {}

    App::new().service(index);
}
