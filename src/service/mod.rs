pub use self::registry::Registry;
use crate::{
    error::Error,
    http::{MessageBody, Req, Resp},
};
use futures::{
    future::{ok, Ready},
    task::{Context, Poll},
};
use rweb_service::{Service, ServiceFactory};
use std::marker::PhantomData;

mod registry;

pub(crate) struct ServiceFactoryWrapper<T> {
    factory: Option<T>,
}

impl<T> ServiceFactoryWrapper<T> {
    pub fn new(factory: T) -> Self {
        Self {
            factory: Some(factory),
        }
    }
}

impl<T> AppServiceFactory for ServiceFactoryWrapper<T>
where
    T: HttpServiceFactory,
{
    fn register(&mut self, config: &mut Registry) {
        if let Some(item) = self.factory.take() {
            item.register(config)
        }
    }
}

pub(crate) trait AppServiceFactory {
    fn register(&mut self, config: &mut Registry);
}

pub trait HttpServiceFactory {
    fn register(self, reg: &mut Registry);
}

#[derive(Debug, Clone, Copy)]
pub struct NoopServiceFactory<T: MessageBody>(pub(crate) PhantomData<T>);

impl<Body> ServiceFactory for NoopServiceFactory<Body>
where
    Body: MessageBody,
{
    type Request = Req;
    type Response = Resp<Body>;
    type Error = Error;
    type Config = ();
    type Service = NoopService<Body>;
    type InitError = ();
    type Future = Ready<Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: Self::Config) -> Self::Future {
        ok(NoopService(PhantomData::default()))
    }
}

pub struct NoopService<Body: MessageBody>(PhantomData<Body>);

impl<Body> Service for NoopService<Body>
where
    Body: MessageBody,
{
    type Request = Req;
    type Response = Resp<Body>;
    type Error = Error;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Self::Request) -> Self::Future {
        unreachable!("NoopService.call should not be caleld")
    }
}
