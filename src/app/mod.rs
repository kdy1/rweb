use crate::{
    data::{Data, DataFactory, FnDataFactory},
    error::Error,
    http::{Body, MessageBody, Req, Resp},
    resource::{HttpNewService, Resource},
    route::Route,
    service::{AppServiceFactory, HttpServiceFactory, NoopServiceFactory, ServiceFactoryWrapper},
};
use futures::FutureExt;
use http::Extensions;
use rweb_service::{apply, boxed, IntoServiceFactory, ServiceFactory, Transform};
use std::{fmt, future::Future, rc::Rc};

mod hyper;

/// An application builder.
pub struct App<S, Body>
where
    S: ServiceFactory<
        Config = (),
        Request = Req,
        Response = Resp<Body>,
        Error = Error,
        InitError = (),
    >,
    Body: MessageBody,
{
    endpoint: S,
    services: Vec<Box<dyn AppServiceFactory>>,
    default: Option<Rc<HttpNewService>>,
    data: Vec<Box<dyn DataFactory>>,
    data_factories: Vec<FnDataFactory>,
    extensions: Extensions,
}

impl App<NoopServiceFactory<Body>, Body> {
    pub fn new() -> Self {
        App {
            endpoint: NoopServiceFactory(Default::default()),
            services: Default::default(),
            default: None,
            data: Default::default(),
            data_factories: Default::default(),
            extensions: Default::default(),
        }
    }
}

impl<S, Body> App<S, Body>
where
    S: ServiceFactory<
        Config = (),
        Request = Req,
        Response = Resp<Body>,
        Error = Error,
        InitError = (),
    >,
    Body: MessageBody,
{
    /// Set application data. Application data could be accessed
    /// by using `Data<T>` extractor where `T` is data type.
    ///
    /// **Note**: http server accepts an application factory rather than
    /// an application instance. Http server constructs an application
    /// instance for each thread, thus application data must be constructed
    /// multiple times. If you want to share data between different
    /// threads, a shared object should be used, e.g. `Arc`. Internally `Data`
    /// type uses `Arc` so data could be created outside of app factory and
    /// clones could be stored via `App::app_data()` method.
    ///
    /// ```rust
    /// use std::cell::Cell;
    /// use actix_web::{web, App, HttpResponse, Responder};
    ///
    /// struct MyData {
    ///     counter: Cell<usize>,
    /// }
    ///
    /// async fn index(data: web::Data<MyData>) -> impl Responder {
    ///     data.counter.set(data.counter.get() + 1);
    ///     HttpResponse::Ok()
    /// }
    ///
    /// let app = App::new()
    ///     .data(MyData{ counter: Cell::new(0) })
    ///     .service(
    ///         web::resource("/index.html").route(
    ///             web::get().to(index)));
    /// ```
    pub fn data<U: 'static + Sync + Send>(mut self, data: U) -> Self {
        self.data.push(Box::new(Data::new(data)));
        self
    }

    /// Set application data factory. This function is
    /// similar to `.data()` but it accepts data factory. Data object get
    /// constructed asynchronously during application initialization.
    pub fn data_factory<F, Out, D, E>(mut self, data: F) -> Self
    where
        F: Fn() -> Out + 'static,
        Out: Future<Output = Result<D, E>> + 'static,
        D: 'static + Sync + Send,
        E: std::fmt::Debug,
    {
        self.data_factories.push(Box::new(move || {
            {
                let fut = data();
                async move {
                    match fut.await {
                        Err(e) => {
                            log::error!("Can not construct data instance: {:?}", e);
                            Err(())
                        }
                        Ok(data) => {
                            let data: Box<dyn DataFactory> = Box::new(Data::new(data));
                            Ok(data)
                        }
                    }
                }
            }
            .boxed_local()
        }));
        self
    }

    /// Set application level arbitrary data item.
    ///
    /// Application data stored with `App::app_data()` method is available
    /// via `HttpRequest::app_data()` method at runtime.
    ///
    /// This method could be used for storing `Data<T>` as well, in that case
    /// data could be accessed by using `Data<T>` extractor.
    pub fn app_data<U: 'static + Send + Sync>(mut self, ext: U) -> Self {
        self.extensions.insert(ext);
        self
    }

    /// Configure route for a specific path.
    ///
    /// This is a simplified version of the `App::service()` method.
    /// This method can be used multiple times with same path, in that case
    /// multiple resources with one route would be registered for same resource
    /// path.
    ///
    /// ```rust
    /// use actix_web::{web, App, HttpResponse};
    ///
    /// async fn index(data: web::Path<(String, String)>) -> &'static str {
    ///     "Welcome!"
    /// }
    ///
    /// fn main() {
    ///     let app = App::new()
    ///         .route("/test1", web::get().to(index))
    ///         .route("/test2", web::post().to(|| HttpResponse::MethodNotAllowed()));
    /// }
    /// ```
    pub fn route(self, path: &str, mut route: Route) -> Self {
        self.service(
            Resource::new(path)
                .add_guards(route.take_guards())
                .route(route),
        )
    }

    /// Register http service.
    ///
    /// Http service is any type that implements `HttpServiceFactory` trait.
    ///
    /// Actix web provides several services implementations:
    ///
    /// * *Resource* is an entry in resource table which corresponds to
    ///   requested URL.
    /// * *Scope* is a set of resources with common root path.
    /// * "StaticFiles" is a service for static files support
    pub fn service<F>(mut self, factory: F) -> Self
    where
        F: HttpServiceFactory + 'static,
    {
        self.services
            .push(Box::new(ServiceFactoryWrapper::new(factory)));
        self
    }

    /// Default service to be used if no matching resource could be found.
    ///
    /// It is possible to use services like `Resource`, `Route`.
    ///
    /// ```rust
    /// use actix_web::{web, App, HttpResponse};
    ///
    /// async fn index() -> &'static str {
    ///     "Welcome!"
    /// }
    ///
    /// fn main() {
    ///     let app = App::new()
    ///         .service(
    ///             web::resource("/index.html").route(web::get().to(index)))
    ///         .default_service(
    ///             web::route().to(|| HttpResponse::NotFound()));
    /// }
    /// ```
    ///
    /// It is also possible to use static files as default service.
    ///
    /// ```rust
    /// use actix_web::{web, App, HttpResponse};
    ///
    /// fn main() {
    ///     let app = App::new()
    ///         .service(
    ///             web::resource("/index.html").to(|| HttpResponse::Ok()))
    ///         .default_service(
    ///             web::to(|| HttpResponse::NotFound())
    ///         );
    /// }
    /// ```
    pub fn default_service<F, U>(mut self, f: F) -> Self
    where
        F: IntoServiceFactory<U>,
        U: ServiceFactory<Config = (), Request = Req, Response = Resp, Error = Error> + 'static,
        U::InitError: fmt::Debug,
    {
        // create and configure default resource
        self.default = Some(Rc::new(boxed::factory(f.into_factory().map_init_err(
            |e| log::error!("Can not construct default service: {:?}", e),
        ))));

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
            Request = Req,
            Response = Resp<B1>,
            Error = Error,
            InitError = (),
        >,
        B1,
    >
    where
        M: Transform<S::Service, Request = Req, Response = Resp<B1>, Error = Error, InitError = ()>,
        B1: MessageBody,
    {
        App {
            endpoint: apply(mw, self.endpoint),
            services: self.services,
            default: self.default,
            data: self.data,
            data_factories: self.data_factories,
            extensions: self.extensions,
        }
    }
}
