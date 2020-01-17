pub trait HttpServiceFactory {
    fn register(self, config: &mut AppService);
}

pub(crate) trait AppServiceFactory {
    fn register(&mut self, config: &mut AppService);
}
