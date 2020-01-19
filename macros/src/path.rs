use pmutil::{q, Quote};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::{
    parse_quote::parse, punctuated::Punctuated, Expr, ItemFn, LitStr, ReturnType, Signature, Token,
};

pub fn compile(path: TokenStream, sig: &Signature) -> (Expr, HashMap<String, String>) {
    let path: LitStr = parse(path);
    let path = path.value();
    assert!(path.starts_with('/'), "Path should start with /");

    let segments = path.split('/');
    let len = segments.clone().filter(|&s| s != "").count();

    if len == 0 {
        return (
            q!({ rweb::filters::path::end() }).parse(),
            Default::default(),
        );
    }

    let mut exprs: Punctuated<Expr, Token![.]> = Default::default();

    for segment in segments {
        let is_empty = exprs.is_empty();

        let expr = if segment.starts_with('{') {
            q!({ rweb::filters::path::param(segment) })
        } else {
            q!(Vars { segment }, { rweb::filters::path::path(segment) })
        };

        if is_empty {
            exprs.push(expr.parse());
        } else {
            exprs.push(q!(Vars { expr }, { and(expr) }).parse());
        }
    }
    exprs.push(q!({ and(rweb::filters::path::end()) }).parse());

    (q!(Vars { exprs }, { exprs }).parse(), Default::default())
}
