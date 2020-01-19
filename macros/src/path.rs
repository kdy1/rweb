use pmutil::q;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::{parse_quote::parse, punctuated::Punctuated, Expr, FnArg, LitStr, Pat, Signature, Token};

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
            let v = &segment[1..segment.len() - 1];

            let ty = sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    FnArg::Typed(ty) => match *ty.pat {
                        Pat::Ident(ref i) if i.ident == v => Some(&ty.ty),
                        _ => None,
                    },

                    _ => None,
                })
                .next()
                .unwrap_or_else(|| panic!("failed to find parameter named {}", v));

            q!(Vars { ty }, { rweb::filters::path::param::<ty>() })
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
