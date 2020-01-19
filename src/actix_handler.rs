use crate::{
    error::Error,
    extract::FromRequest,
    http::{Req, Resp},
    responder::Responder,
};
use futures::{
    future::{ok, Ready},
    ready,
};
use pin_project::pin_project;
use rweb_service::{Service, ServiceFactory};
use std::{
    convert::Infallible,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

/// Async handler converter factory
pub trait Factory<T, R, O>: Clone + 'static
where
    R: Future<Output = O>,
    O: Responder,
{
    fn call(&self, param: T) -> R;
}

impl<F, R, O> Factory<(), R, O> for F
where
    F: Fn() -> R + Clone + 'static,
    R: Future<Output = O>,
    O: Responder,
{
    fn call(&self, _: ()) -> R {
        (self)()
    }
}

#[doc(hidden)]
pub struct Handler<F, T, R, O>
where
    F: Factory<T, R, O>,
    R: Future<Output = O>,
    O: Responder,
{
    hnd: F,
    _t: PhantomData<(T, R, O)>,
}

impl<F, T, R, O> Handler<F, T, R, O>
where
    F: Factory<T, R, O>,
    R: Future<Output = O>,
    O: Responder,
{
    pub fn new(hnd: F) -> Self {
        Handler {
            hnd,
            _t: PhantomData,
        }
    }
}

impl<F, T, R, O> Clone for Handler<F, T, R, O>
where
    F: Factory<T, R, O>,
    R: Future<Output = O>,
    O: Responder,
{
    fn clone(&self) -> Self {
        Handler {
            hnd: self.hnd.clone(),
            _t: PhantomData,
        }
    }
}

impl<F, T, R, O> Service for Handler<F, T, R, O>
where
    F: Factory<T, R, O>,
    R: Future<Output = O>,
    O: Responder,
{
    type Request = (T, Req);
    type Response = Resp;
    type Error = Infallible;
    type Future = HandlerServiceResponse<R, O>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, (param, req): (T, Req)) -> Self::Future {
        HandlerServiceResponse {
            fut: self.hnd.call(param),
            fut2: None,
            req: Some(req),
        }
    }
}

#[doc(hidden)]
#[pin_project]
pub struct HandlerServiceResponse<T, R>
where
    T: Future<Output = R>,
    R: Responder,
{
    #[pin]
    fut: T,
    #[pin]
    fut2: Option<R::Future>,
    req: Option<Req>,
}

impl<T, R> Future for HandlerServiceResponse<T, R>
where
    T: Future<Output = R>,
    R: Responder,
{
    type Output = Result<Resp, Infallible>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();

        if let Some(fut) = this.fut2.as_pin_mut() {
            return match fut.poll(cx) {
                Poll::Ready(Ok(res)) => Poll::Ready(Ok(res)),
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => {
                    let res: Resp = Resp::from_err(e.into(), this.req.take().unwrap());
                    Poll::Ready(Ok(res))
                }
            };
        }

        match this.fut.poll(cx) {
            Poll::Ready(res) => {
                let fut = res.respond_to(this.req.as_ref().unwrap());
                self.as_mut().project().fut2.set(Some(fut));
                self.poll(cx)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Extract arguments from request
pub struct Extract<T: FromRequest, S> {
    service: S,
    _t: PhantomData<T>,
}

impl<T: FromRequest, S> Extract<T, S> {
    pub fn new(service: S) -> Self {
        Extract {
            service,
            _t: PhantomData,
        }
    }
}

impl<T: FromRequest, S> ServiceFactory for Extract<T, S>
where
    S: Service<Request = (T, Req), Response = Resp, Error = Infallible> + Clone,
{
    type Request = Req;
    type Response = Resp;
    type Error = (Error, Req);
    type Config = ();
    type Service = ExtractService<T, S>;
    type InitError = ();
    type Future = Ready<Result<Self::Service, ()>>;

    fn new_service(&self, _: ()) -> Self::Future {
        ok(ExtractService {
            _t: PhantomData,
            service: self.service.clone(),
        })
    }
}

pub struct ExtractService<T: FromRequest, S> {
    service: S,
    _t: PhantomData<T>,
}

impl<T: FromRequest, S> Service for ExtractService<T, S>
where
    S: Service<Request = (T, Req), Response = Resp, Error = Infallible> + Clone,
{
    type Request = Req;
    type Response = Resp;
    type Error = (Error, Req);
    type Future = ExtractResponse<T, S>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let fut = T::extract(&req);

        ExtractResponse {
            fut,
            req,
            fut_s: None,
            service: self.service.clone(),
        }
    }
}

#[pin_project]
pub struct ExtractResponse<T: FromRequest, S: Service> {
    req: Req,
    service: S,
    #[pin]
    fut: T::Future,
    #[pin]
    fut_s: Option<S::Future>,
}

impl<T: FromRequest, S> Future for ExtractResponse<T, S>
where
    S: Service<Request = (T, Req), Response = Resp, Error = Infallible>,
{
    type Output = Result<Resp, (Error, Req)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().project();

        if let Some(fut) = this.fut_s.as_pin_mut() {
            return fut.poll(cx).map_err(|_| panic!());
        }

        match ready!(this.fut.poll(cx)) {
            Err(e) => {
                let req = this.req.clone();
                Poll::Ready(Err((e.into(), req)))
            }
            Ok(item) => {
                let fut = Some(this.service.call((item, this.req.clone())));
                self.as_mut().project().fut_s.set(fut);
                self.poll(cx)
            }
        }
    }
}

/// FromRequest trait impl for tuples
macro_rules! factory_tuple ({ $(($n:tt, $T:ident)),+} => {
    impl<Func, $($T,)+ Res, O> Factory<($($T,)+), Res, O> for Func
    where Func: Fn($($T,)+) -> Res + Clone + 'static,
          Res: Future<Output = O>,
          O: Responder,
    {
        fn call(&self, param: ($($T,)+)) -> Res {
            (self)($(param.$n,)+)
        }
    }
});

#[rustfmt::skip]
mod m {
    use super::*;

    factory_tuple!((0, A));
    factory_tuple!((0, A), (1, B));
    factory_tuple!((0, A), (1, B), (2, C));
    factory_tuple!((0, A), (1, B), (2, C), (3, D));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I));
    factory_tuple!((0, A), (1, B), (2, C), (3, D), (4, E), (5, F), (6, G), (7, H), (8, I), (9, J));
}
