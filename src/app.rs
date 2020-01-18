use crate::{
    error::Error,
    service::{AppServiceFactory, HttpServiceFactory},
};
use hyper::{service::HttpService, Request};
use rweb_service::{Service, ServiceFactory, Transform};
use std::process::Output;

/// An application builder.
#[derive(Default)]
pub struct App<S, Body>
where
    S: HttpService<Body>,
{
    endpoint: S,
    services: Vec<Box<dyn AppServiceFactory>>,
}

impl<S, Body> App<S, Body>
where
    S: ServiceFactory<
        Config = (),
        Request = ServiceRequest,
        Response = ServiceResponse<B>,
        Error = Error,
        InitError = (),
    >,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn service(mut self, svc: impl 'static + AppServiceFactory) -> Self {
        self.services.push(Box::new(svc));
        self
    }

    /// Registers middleware, in the form of a middleware component (type),
    /// that runs during inbound and/or outbound processing in the request
    /// lifecycle (request -> response), modifying request/response as
    /// necessary, across all requests managed by the *Application*.
    ///
    /// Use middleware when you need to read or modify *every* request or
    /// response in some way.
    ///
    /// Notice that the keyword for registering middleware is `wrap`. As you
    /// register middleware using `wrap` in the App builder,  imagine wrapping
    /// layers around an inner App.  The first middleware layer exposed to a
    /// Request is the outermost layer-- the *last* registered in
    /// the builder chain.  Consequently, the *first* middleware registered
    /// in the builder chain is the *last* to execute during request processing.
    ///
    /// ```rust
    /// use rweb_service::Service;
    /// use rweb::App;
    /// use rweb::http::{header::CONTENT_TYPE, HeaderValue};
    ///
    /// async fn index() -> &'static str {
    ///     "Welcome!"
    /// }
    ///
    /// fn main() {
    ///     let app = App::new()
    ///         .wrap(middleware::Logger::default())
    ///         .route("/index.html", web::get().to(index));
    /// }
    /// ```
    pub fn wrap<M, B1>(
        self,
        mw: M,
    ) -> App<
        impl ServiceFactory<
            Config = (),
            Request = ServiceRequest,
            Response = ServiceResponse<B1>,
            Error = Error,
            InitError = (),
        >,
        B1,
    >
    where
        M: Transform<
            S::Service,
            Request = ServiceRequest,
            Response = ServiceResponse<B1>,
            Error = Error,
            InitError = (),
        >,
        B1: MessageBody,
    {
        App {
            endpoint: apply(mw, self.endpoint),
            services: self.services,
        }
    }
}
