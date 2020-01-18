use crate::{
    error::Error,
    http::{Req, Resp},
};
use rweb_service::boxed::BoxServiceFactory;

pub(crate) type HttpNewService = BoxServiceFactory<(), Req, Resp, Error, ()>;
