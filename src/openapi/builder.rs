use super::*;

/// Builder for openapi v3 specification.
#[derive(Debug, Clone, Default)]
pub struct Builder {
    spec: Spec,
}

/// Crates a new specification builder
#[inline(always)]
pub fn spec() -> Builder {
    Builder::default()
}

impl Builder {
    #[inline(always)]
    pub fn info(mut self, info: Info) -> Self {
        self.spec.info = info;
        self
    }

    /// Creates an openapi specification. You can serialize this as json or yaml
    /// to generate client codes.
    pub fn build<F, Ret>(self, op: F) -> (Spec, Ret)
    where
        F: FnOnce() -> Ret,
    {
        let mut collector = new();
        collector.spec = self.spec;

        let cell = RefCell::new(collector);

        let ret = COLLECTOR.set(&cell, || op());
        let mut spec = cell.into_inner().spec;
        spec.openapi = "3.0.1".into();
        (spec, ret)
    }
}
