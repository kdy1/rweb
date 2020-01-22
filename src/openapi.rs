use rweb_openapi::v3_0::Spec;
use std::sync::{Arc, Mutex};
use warp::{Filter, Reply};

#[derive(Debug, Clone)]
pub struct Collector {
    spec: Arc<Mutex<Spec>>,
}

pub async fn spec<F>(f: &F) -> Spec
where
    F: 'static + Filter,
    F::Extract: Reply,
{
    let collector = Collector {
        spec: Default::default(),
    };

    let req = warp::test::request().extension::<Collector>(collector.clone());
    let _ = req.reply(f).await;

    let spec = Arc::try_unwrap(collector.spec).expect("failed to unwrap Arc in Arc<Mutex<Spec>>");
    let spec = Mutex::into_inner(spec).expect("failed to unwrap Mutex in Mutex<Spec>");

    spec
}
