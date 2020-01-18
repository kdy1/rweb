use crate::{
    error::Error,
    guard::Guard,
    http::{Req, Resp},
};
use rweb_service::boxed::BoxServiceFactory;
use std::{cell::RefCell, rc::Rc};

pub(crate) type HttpNewService = BoxServiceFactory<(), Req, Resp, Error, ()>;
