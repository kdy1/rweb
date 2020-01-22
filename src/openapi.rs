pub use self::schema::*;
use openapi::Spec;
use rweb_openapi::v3_0::Spec;
use warp::Filter;

#[derive(Debug)]
pub struct Collector {
    spec: Spec,
}

pub async fn spec(f: &dyn Filter) -> Spec {
    let collector = Collector { spec: Spec };

    let req = warp::test::request().extension::<Collector>(collector);
    let res = req.reply(f).await;

    let (mut head, _) = res.into_parts();
    let collector = head
        .extensions
        .remove::<Collector>()
        .expect("Is collector removed?");

    collector.spec
}
