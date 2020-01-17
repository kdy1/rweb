use actix_service::ServiceFactory;

/// An application builder.
#[derive(Default)]
pub struct App<Entry, Body>
where
    Entry: ServiceFactory<Config = ()>,
{
    services: Vec<Box<dyn AppServiceFactory>>,
}

impl<Entry, Body> App<Entry, Body>
where
    Entry: ServiceFactoy<Config = ()>,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn service(mut self, svc: impl 'static + HttpServiceFactory) -> Self {
        self.services.push(Box::new(svc));
        self
    }

    /// Applies a middleware
    pub fn wrap<M>(self, middleware: M) -> App {}
}
