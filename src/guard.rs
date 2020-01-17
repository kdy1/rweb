use crate::http::ReqInfo;

/// Guards can prevent route from matching.
pub trait Guard {
    fn allow(&self, req: &ReqInfo) -> bool;
}

#[derive(Debug, Default, Copy, Clone)]
pub struct AllowAll;

impl Guard for AllowAll {
    fn allow(&self, _: &ReqInfo) -> bool {
        true
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct DenyAll;

impl Guard for DenyAll {
    fn allow(&self, _: &ReqInfo) -> bool {
        false
    }
}

impl<F> Guard for F
where
    F: Fn(&ReqInfo) -> bool,
{
    fn allow(&self, req: &ReqInfo) -> bool {
        (self)(req)
    }
}
