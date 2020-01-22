use rweb_openapi::v3_0::Spec;
use warp::{Filter, Reply};

#[derive(Debug)]
pub struct Collector {
    spec: Spec,
}

pub async fn spec<F>(f: &F) -> Spec
where
    F: 'static + Filter,
    F::Extract: Reply,
{
    let collector = Collector {
        spec: Spec::default(),
    };

    let req = warp::test::request().extension::<Collector>(collector);
    let res = req.reply(f).await;

    let (mut head, _) = res.into_parts();
    let collector = head
        .extensions
        .remove::<Collector>()
        .expect("Is collector removed?");

    collector.spec
}
