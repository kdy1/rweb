use rweb_openapi::v3_0::Spec;
use scoped_tls::scoped_thread_local;
use std::cell::RefCell;

scoped_thread_local!(static COLLECTOR: RefCell<Collector>);

#[derive(Debug)]
pub struct Collector {
    spec: Spec,
    path_prefix: String,
}

pub fn collect<F>(op: F) -> Spec
where
    F: FnOnce(),
{
    let collector = Collector {
        spec: Default::default(),
        path_prefix: Default::default(),
    };

    let cell = RefCell::new(collector);

    COLLECTOR.set(&cell, || op());

    cell.into_inner().spec
}

pub fn with<F>(op: F)
where
    F: FnOnce(&mut Collector),
{
    if COLLECTOR.is_set() {
        COLLECTOR.with(|c| {
            let mut r = c.borrow_mut();
            op(&mut r);
        });
    }
}
