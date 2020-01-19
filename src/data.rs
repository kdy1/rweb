use crate::{
    error::{Error, ErrorInternalServerError},
    extract::FromRequest,
    http::Payload,
    Req,
};
use futures::future::{err, ok, Ready};
use http::Extensions;
use std::{ops::Deref, sync::Arc};

/// Application data factory
pub(crate) trait DataFactory {
    fn create(&self, extensions: &mut Extensions) -> bool;
}

/// Application data.
///
/// Application data is an arbitrary data attached to the app.
/// Application data is available to all routes and could be added
/// during application configuration process
/// with `App::data()` method.
///
/// Application data could be accessed by using `Data<T>`
/// extractor where `T` is data type.
///
/// **Note**: http server accepts an application factory rather than
/// an application instance. Http server constructs an application
/// instance for each thread, thus application data must be constructed
/// multiple times. If you want to share data between different
/// threads, a shareable object should be used, e.g. `Send + Sync`. Application
/// data does not need to be `Send` or `Sync`. Internally `Data` type
/// uses `Arc`. if your data implements `Send` + `Sync` traits you can
/// use `web::Data::new()` and avoid double `Arc`.
///
/// If route data is not set for a handler, using `Data<T>` extractor would
/// cause *Internal Server Error* response.
///
/// ```rust
/// use std::sync::Mutex;
/// use actix_web::{web, App, HttpResponse, Responder};
///
/// struct MyData {
///     counter: usize,
/// }
///
/// /// Use `Data<T>` extractor to access data in handler.
/// async fn index(data: web::Data<Mutex<MyData>>) -> impl Responder {
///     let mut data = data.lock().unwrap();
///     data.counter += 1;
///     HttpResponse::Ok()
/// }
///
/// fn main() {
///     let data = web::Data::new(Mutex::new(MyData{ counter: 0 }));
///
///     let app = App::new()
///         // Store `MyData` in application storage.
///         .app_data(data.clone())
///         .service(
///             web::resource("/index.html").route(
///                 web::get().to(index)));
/// }
/// ```
#[derive(Debug)]
pub struct Data<T>(Arc<T>);

impl<T> Data<T> {
    /// Create new `Data` instance.
    ///
    /// Internally `Data` type uses `Arc`. if your data implements
    /// `Send` + `Sync` traits you can use `web::Data::new()` and
    /// avoid double `Arc`.
    pub fn new(state: T) -> Data<T> {
        Data(Arc::new(state))
    }

    /// Get reference to inner app data.
    pub fn get_ref(&self) -> &T {
        self.0.as_ref()
    }

    /// Convert to the internal Arc<T>
    pub fn into_inner(self) -> Arc<T> {
        self.0
    }
}

impl<T> Deref for Data<T> {
    type Target = Arc<T>;

    fn deref(&self) -> &Arc<T> {
        &self.0
    }
}

impl<T> Clone for Data<T> {
    fn clone(&self) -> Data<T> {
        Data(self.0.clone())
    }
}

impl<T: 'static> FromRequest for Data<T>
where
    T: Send + Sync,
{
    type Error = Error;
    type Future = Ready<Result<Self, Error>>;
    type Config = ();

    #[inline]
    fn from_request(req: &Req, _: &mut Payload) -> Self::Future {
        if let Some(st) = req.app_data::<Data<T>>() {
            ok(st.clone())
        } else {
            log::debug!(
                "Failed to construct App-level Data extractor. Request path: {:?}",
                req.path()
            );
            err(ErrorInternalServerError(
                "App data is not configured, to configure use App::data()",
            ))
        }
    }
}

impl<T> DataFactory for Data<T>
where
    T: 'static + Sync + Send,
{
    fn create(&self, extensions: &mut Extensions) -> bool {
        if extensions.get::<Data<T>>().is_none() {
            extensions.insert(Data(self.0.clone()));
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_service::Service;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;
    use crate::{
        http::StatusCode,
        test::{self, init_service, TestRequest},
        web, App, HttpResponse,
    };

    #[tokio::test]
    async fn test_data_extractor() {
        let mut srv = init_service(App::new().data("TEST".to_string()).service(
            web::resource("/").to(|data: web::Data<String>| {
                assert_eq!(data.to_lowercase(), "test");
                HttpResponse::Ok()
            }),
        ))
        .await;

        let req = TestRequest::default().to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let mut srv = init_service(
            App::new()
                .data(10u32)
                .service(web::resource("/").to(|_: web::Data<usize>| HttpResponse::Ok())),
        )
        .await;
        let req = TestRequest::default().to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_app_data_extractor() {
        let mut srv = init_service(
            App::new()
                .app_data(Data::new(10usize))
                .service(web::resource("/").to(|_: web::Data<usize>| HttpResponse::Ok())),
        )
        .await;

        let req = TestRequest::default().to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let mut srv = init_service(
            App::new()
                .app_data(Data::new(10u32))
                .service(web::resource("/").to(|_: web::Data<usize>| HttpResponse::Ok())),
        )
        .await;
        let req = TestRequest::default().to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_route_data_extractor() {
        let mut srv = init_service(App::new().service(web::resource("/").data(10usize).route(
            web::get().to(|data: web::Data<usize>| {
                let _ = data.clone();
                HttpResponse::Ok()
            }),
        )))
        .await;

        let req = TestRequest::default().to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // different type
        let mut srv = init_service(
            App::new().service(
                web::resource("/")
                    .data(10u32)
                    .route(web::get().to(|_: web::Data<usize>| HttpResponse::Ok())),
            ),
        )
        .await;
        let req = TestRequest::default().to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_override_data() {
        let mut srv = init_service(
            App::new()
                .data(1usize)
                .service(web::resource("/").data(10usize).route(web::get().to(
                    |data: web::Data<usize>| {
                        assert_eq!(**data, 10);
                        let _ = data.clone();
                        HttpResponse::Ok()
                    },
                ))),
        )
        .await;

        let req = TestRequest::default().to_request();
        let resp = srv.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_data_drop() {
        struct TestData(Arc<AtomicUsize>);

        impl TestData {
            fn new(inner: Arc<AtomicUsize>) -> Self {
                let _ = inner.fetch_add(1, Ordering::SeqCst);
                Self(inner)
            }
        }

        impl Clone for TestData {
            fn clone(&self) -> Self {
                let inner = self.0.clone();
                let _ = inner.fetch_add(1, Ordering::SeqCst);
                Self(inner)
            }
        }

        impl Drop for TestData {
            fn drop(&mut self) {
                let _ = self.0.fetch_sub(1, Ordering::SeqCst);
            }
        }

        let num = Arc::new(AtomicUsize::new(0));
        let data = TestData::new(num.clone());
        assert_eq!(num.load(Ordering::SeqCst), 1);

        let srv = test::start(move || {
            let data = data.clone();

            App::new()
                .data(data)
                .service(web::resource("/").to(|_data: Data<TestData>| async { "ok" }))
        });

        assert!(srv.get("/").send().await.unwrap().status().is_success());
        srv.stop().await;

        assert_eq!(num.load(Ordering::SeqCst), 0);
    }
}
