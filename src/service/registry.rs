use crate::{error::Error, guard::Guards, resource::HttpNewService, rmap::ResourceMap, Req, Resp};
use rweb_router::ResourceDef;
use rweb_service::{boxed, IntoServiceFactory, ServiceFactory};
use std::rc::Rc;

/// Service registry
pub struct Registry {
    //    config: AppConfig,
    root: bool,
    default: Rc<HttpNewService>,
    services: Vec<Registered>,
}

impl Registry {
    pub fn is_root(&self) -> bool {
        self.root
    }

    /// Register http service
    pub fn register_service<F, S>(
        &mut self,
        rdef: ResourceDef,
        guards: Option<Guards>,
        factory: F,
        nested: Option<Rc<ResourceMap>>,
    ) where
        F: IntoServiceFactory<S>,
        S: ServiceFactory<
                Config = (),
                Request = Req,
                Response = Resp,
                Error = Error,
                InitError = (),
            > + 'static,
    {
        self.services.push(Registered {
            def: rdef,
            svc: boxed::factory(factory.into_factory()),
            guards,
            resource_maps: nested,
        });
    }
}

/// Represents a registered service.
struct Registered {
    def: ResourceDef,
    svc: HttpNewService,
    guards: Option<Guards>,
    resource_maps: Option<Rc<ResourceMap>>,
}
