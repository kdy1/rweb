use http::StatusCode;
use rweb::{filters::BoxedFilter, *};

impl FromRequest for User {
    type Filter = BoxedFilter<(User,)>;

    fn new() -> Self::Filter {
        let header = header::<String>("x-user-id");
        header.map(|id| User { id }).boxed()
    }
}

struct User {
    id: String,
}

#[get("/")]
fn index(user: User) -> String {
    user.id
}

#[tokio::test]
async fn index_test() {
    let value = warp::test::request()
        .header("x-user-id", "test-uid")
        .path("/")
        .reply(&index())
        .await;

    assert_eq!(value.status(), StatusCode::OK);
    assert_eq!(value.into_body(), b"test-uid"[..]);
}

#[tokio::test]
async fn index_test_fail() {
    let value = warp::test::request().path("/").reply(&index()).await;

    assert_eq!(value.status(), StatusCode::BAD_REQUEST);
}
