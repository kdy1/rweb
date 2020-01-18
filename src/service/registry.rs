use crate::{guard::Guards, resource::HttpNewService, rmap::ResourceMap};
use rweb_router::ResourceDef;
use std::rc::Rc;

/// Service registry
pub struct Registry {
    //    config: AppConfig,
    root: bool,
    default: Rc<HttpNewService>,
    services: Vec<Registered>,
}

/// Represents a registered service.
struct Registered {
    def: ResourceDef,
    svc: HttpNewService,
    guards: Option<Guards>,
    resource_maps: Option<Rc<ResourceMap>>,
}
