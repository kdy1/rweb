use crate::{
    error::Error,
    extract::FromRequest,
    guard::{self, Guard},
    handler::{Extract, Factory, Handler},
    responder::Responder,
    Req, Resp,
};
use futures::future::{ready, FutureExt, LocalBoxFuture};
use hyper::Method;
use rweb_service::{Service, ServiceFactory};
use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
};

type BoxedRouteService<Req, Res> = Box<
    dyn Service<
        Request = Req,
        Response = Res,
        Error = Error,
        Future = LocalBoxFuture<'static, Result<Res, Error>>,
    >,
>;

type BoxedRouteNewService<Req, Res> = Box<
    dyn ServiceFactory<
        Config = (),
        Request = Req,
        Response = Res,
        Error = Error,
        InitError = (),
        Service = BoxedRouteService<Req, Res>,
        Future = LocalBoxFuture<'static, Result<BoxedRouteService<Req, Res>, ()>>,
    >,
>;

/// Resource route definition
///
/// Route uses builder-like pattern for configuration.
/// If handler is not explicitly set, default *404 Not Found* handler is used.
pub struct Route {
    service: BoxedRouteNewService<Req, Resp>,
    guards: Rc<Vec<Box<dyn Guard>>>,
}

impl Route {
    /// Create new route which matches any request.
    pub fn new() -> Route {
        Route {
            service: Box::new(RouteNewService::new(Extract::new(Handler::new(|| {
                ready(Resp::not_found().build())
            })))),
            guards: Rc::new(Vec::new()),
        }
    }

    pub(crate) fn take_guards(&mut self) -> Vec<Box<dyn Guard>> {
        std::mem::replace(Rc::get_mut(&mut self.guards).unwrap(), Vec::new())
    }
}

impl ServiceFactory for Route {
    type Request = Req;
    type Response = Resp;
    type Error = Error;
    type Config = ();
    type Service = RouteService;
    type InitError = ();
    type Future = CreateRouteService;

    fn new_service(&self, _: ()) -> Self::Future {
        CreateRouteService {
            fut: self.service.new_service(()),
            guards: self.guards.clone(),
        }
    }
}

type RouteFuture = LocalBoxFuture<'static, Result<BoxedRouteService<Req, Resp>, ()>>;

#[pin_project::pin_project]
pub struct CreateRouteService {
    #[pin]
    fut: RouteFuture,
    guards: Rc<Vec<Box<dyn Guard>>>,
}

impl Future for CreateRouteService {
    type Output = Result<RouteService, ()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.fut.poll(cx)? {
            Poll::Ready(service) => Poll::Ready(Ok(RouteService {
                service,
                guards: this.guards.clone(),
            })),
            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct RouteService {
    service: BoxedRouteService<Req, Resp>,
    guards: Rc<Vec<Box<dyn Guard>>>,
}

impl RouteService {
    pub fn check(&self, req: &mut Req) -> bool {
        for f in self.guards.iter() {
            if !f.allow(req.head()) {
                return false;
            }
        }
        true
    }
}

impl Service for RouteService {
    type Request = Req;
    type Response = Resp;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        self.service.call(req).boxed_local()
    }
}

impl Route {
    /// Add method guard to the route.
    ///
    /// ```rust
    /// # use rweb::*;
    /// # fn main() {
    /// App::new().service(web::resource("/path").route(
    ///     web::get()
    ///         .method(http::Method::CONNECT)
    ///         .guard(guard::Header("content-type", "text/plain"))
    ///         .to(|req: HttpRequest| Resp::Ok()))
    /// );
    /// # }
    /// ```
    pub fn method(mut self, method: Method) -> Self {
        Rc::get_mut(&mut self.guards)
            .unwrap()
            .push(Box::new(guard::http::method(method)));
        self
    }

    /// Add guard to the route.
    ///
    /// ```rust
    /// # use rweb::*;
    /// # fn main() {
    /// App::new().service(web::resource("/path").route(
    ///     web::route()
    ///         .guard(guard::Get())
    ///         .guard(guard::Header("content-type", "text/plain"))
    ///         .to(|req: HttpRequest| Resp::Ok()))
    /// );
    /// # }
    /// ```
    pub fn guard<F: Guard + 'static>(mut self, f: F) -> Self {
        Rc::get_mut(&mut self.guards).unwrap().push(Box::new(f));
        self
    }

    /// Set handler function, use request extractors for parameters.
    ///
    /// ```rust
    /// use rweb::{web, http, App};
    /// use serde_derive::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Info {
    ///     username: String,
    /// }
    ///
    /// /// extract path info using serde
    /// async fn index(info: web::Path<Info>) -> String {
    ///     format!("Welcome {}!", info.username)
    /// }
    ///
    /// fn main() {
    ///     let app = App::new().service(
    ///         web::resource("/{username}/index.html") // <- define path parameters
    ///             .route(web::get().to(index))        // <- register handler
    ///     );
    /// }
    /// ```
    ///
    /// It is possible to use multiple extractors for one handler function.
    ///
    /// ```rust
    /// # use std::collections::HashMap;
    /// # use serde_derive::Deserialize;
    /// use rweb::{web, App};
    ///
    /// #[derive(Deserialize)]
    /// struct Info {
    ///     username: String,
    /// }
    ///
    /// /// extract path info using serde
    /// async fn index(path: web::Path<Info>, query: web::Query<HashMap<String, String>>, body: web::Json<Info>) -> String {
    ///     format!("Welcome {}!", path.username)
    /// }
    ///
    /// fn main() {
    ///     let app = App::new().service(
    ///         web::resource("/{username}/index.html") // <- define path parameters
    ///             .route(web::get().to(index))
    ///     );
    /// }
    /// ```
    pub fn to<F, T, R, U>(mut self, handler: F) -> Self
    where
        F: Factory<T, R, U>,
        T: FromRequest + 'static,
        R: Future<Output = U> + 'static,
        U: Responder + 'static,
    {
        self.service = Box::new(RouteNewService::new(Extract::new(Handler::new(handler))));
        self
    }
}

struct RouteNewService<T>
where
    T: ServiceFactory<Request = Req, Error = (Error, Req)>,
{
    service: T,
}

impl<T> RouteNewService<T>
where
    T: ServiceFactory<Config = (), Request = Req, Response = Resp, Error = (Error, Req)>,
    T::Future: 'static,
    T::Service: 'static,
    <T::Service as Service>::Future: 'static,
{
    pub fn new(service: T) -> Self {
        RouteNewService { service }
    }
}

impl<T> ServiceFactory for RouteNewService<T>
where
    T: ServiceFactory<Config = (), Request = Req, Response = Resp, Error = (Error, Req)>,
    T::Future: 'static,
    T::Service: 'static,
    <T::Service as Service>::Future: 'static,
{
    type Config = ();
    type Request = Req;
    type Response = Resp;
    type Error = Error;
    type InitError = ();
    type Service = BoxedRouteService<Req, Self::Response>;
    type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        self.service
            .new_service(())
            .map(|result| match result {
                Ok(service) => {
                    let service: BoxedRouteService<_, _> =
                        Box::new(RouteServiceWrapper { service });
                    Ok(service)
                }
                Err(_) => Err(()),
            })
            .boxed_local()
    }
}

struct RouteServiceWrapper<T: Service> {
    service: T,
}

impl<T> Service for RouteServiceWrapper<T>
where
    T::Future: 'static,
    T: Service<Request = Req, Response = Resp, Error = (Error, Req)>,
{
    type Request = Req;
    type Response = Resp;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx).map_err(|(e, _)| e)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        // let mut fut = self.service.call(req);
        self.service
            .call(req)
            .map(|res| match res {
                Ok(res) => Ok(res),
                Err((err, req)) => Ok(req.error_response(err)),
            })
            .boxed_local()

        // match fut.poll() {
        //     Poll::Ready(Ok(res)) => Either::Left(ok(res)),
        //     Poll::Ready(Err((e, req))) =>
        // Either::Left(ok(req.error_response(e))),     Poll::Pending =>
        // Either::Right(Box::new(fut.then(|res| match res {
        //         Ok(res) => Ok(res),
        //         Err((err, req)) => Ok(req.error_response(err)),
        //     }))),
        // }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use actix_rt::time::delay_for;
    use bytes::Bytes;
    use serde_derive::Serialize;

    use crate::{
        error,
        http::{Method, StatusCode},
        test::{call_service, init_service, read_body, TestRequest},
        web, App, Resp,
    };

    #[derive(Serialize, PartialEq, Debug)]
    struct MyObject {
        name: String,
    }

    #[actix_rt::test]
    async fn test_route() {
        let mut srv = init_service(
            App::new()
                .service(
                    web::resource("/test")
                        .route(web::get().to(|| Resp::Ok()))
                        .route(
                            web::put()
                                .to(|| async { Err::<Resp, _>(error::ErrorBadRequest("err")) }),
                        )
                        .route(web::post().to(|| async {
                            delay_for(Duration::from_millis(100)).await;
                            Resp::Created()
                        }))
                        .route(web::delete().to(|| async {
                            delay_for(Duration::from_millis(100)).await;
                            Err::<Resp, _>(error::ErrorBadRequest("err"))
                        })),
                )
                .service(web::resource("/json").route(web::get().to(|| async {
                    delay_for(Duration::from_millis(25)).await;
                    web::Json(MyObject {
                        name: "test".to_string(),
                    })
                }))),
        )
        .await;

        let req = TestRequest::with_uri("/test")
            .method(Method::GET)
            .to_request();
        let resp = call_service(&mut srv, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let req = TestRequest::with_uri("/test")
            .method(Method::POST)
            .to_request();
        let resp = call_service(&mut srv, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let req = TestRequest::with_uri("/test")
            .method(Method::PUT)
            .to_request();
        let resp = call_service(&mut srv, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let req = TestRequest::with_uri("/test")
            .method(Method::DELETE)
            .to_request();
        let resp = call_service(&mut srv, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let req = TestRequest::with_uri("/test")
            .method(Method::HEAD)
            .to_request();
        let resp = call_service(&mut srv, req).await;
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);

        let req = TestRequest::with_uri("/json").to_request();
        let resp = call_service(&mut srv, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = read_body(resp).await;
        assert_eq!(body, Bytes::from_static(b"{\"name\":\"test\"}"));
    }
}
