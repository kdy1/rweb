//! A macro to convert a function to rweb handler.
//!
//! # Attribute on parameters
//!
//! ## `#[body]`
//! Parses request body
//
//! ## `#[form]`
//! Parses request body
//
//! ## `#[json]`
//! Parses request body.
//!
//! ## `#[query]`
//! Parses query string.

extern crate proc_macro;
use pmutil::{q, smart_quote, Quote, ToTokensExt};
use proc_macro2::{Ident, TokenStream};
use std::collections::HashSet;
use syn::{
    parse_quote::parse,
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    Attribute, Expr, FnArg, ItemFn, Pat, ReturnType, Signature, Token, Visibility,
};

mod path;
mod router;

#[proc_macro_attribute]
pub fn get(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_http_method(q!({ get }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn post(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_http_method(q!({ post }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn put(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_http_method(q!({ put }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn delete(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_http_method(q!({ delete }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn head(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_http_method(q!({ head }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn options(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_http_method(q!({ options }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn patch(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_http_method(q!({ patch }), path.into(), fn_item.into())
}

/// Creates a router. Useful for modularizing codes.
///
///
/// # Note
///
/// Currently router returns 404 error if there is a no matching rule.
#[proc_macro_attribute]
pub fn router(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    router::router(attr.into(), item.into()).dump().into()
}

fn expand_http_method(method: Quote, path: TokenStream, f: TokenStream) -> proc_macro::TokenStream {
    let f: ItemFn = parse(f);
    let sig = &f.sig;
    let block = &f.block;

    // Apply method filter
    let expr: Expr = q!(
        Vars {
            http_method: method,
        },
        { rweb::filters::method::http_method() }
    )
    .parse();

    let (mut expr, vars) = path::compile(Some(expr), path, Some(sig), true);

    let handler_fn = {
        let mut inputs: Punctuated<FnArg, _> = f.sig.inputs.clone();

        {
            // Handle path parameters

            let mut done = HashSet::new();

            for (orig_idx, (name, idx)) in vars.into_iter().enumerate() {
                if orig_idx == idx || done.contains(&orig_idx) {
                    continue;
                }

                match &f.sig.inputs[idx] {
                    FnArg::Typed(pat) => match *pat.pat {
                        Pat::Ident(ref i) if i.ident == name => {
                            inputs[orig_idx] = f.sig.inputs[idx].clone();
                            inputs[idx] = f.sig.inputs[orig_idx].clone();
                            done.insert(idx);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        {
            // Handle annotated parameters.
            for i in inputs.pairs_mut() {
                match i.into_value() {
                    FnArg::Receiver(_) => continue,
                    FnArg::Typed(pat) => {
                        if pat.attrs.is_empty() {
                            continue;
                        }

                        let is_rweb_attr = pat.attrs.iter().any(is_rweb_attr);
                        if !is_rweb_attr {
                            // We don't care about this parameter.
                            continue;
                        }

                        if pat.attrs.len() != 1 {
                            // TODO: Support cfg?
                            panic!("rweb currently support only one attribute on a parameter")
                        }

                        let attr = pat.attrs.iter().next().unwrap().clone();
                        pat.attrs = vec![];

                        if attr.path.is_ident("form") {
                            expr =
                                q!(Vars { expr }, { expr.and(rweb::filters::body::form()) }).parse()
                        } else if attr.path.is_ident("json") {
                            expr =
                                q!(Vars { expr }, { expr.and(rweb::filters::body::json()) }).parse()
                        } else if attr.path.is_ident("body") {
                            expr = q!(Vars { expr }, { expr.and(rweb::filters::body::bytes()) })
                                .parse()
                        } else if attr.path.is_ident("query") {
                            expr = q!(Vars { expr }, { expr.and(rweb::filters::query::query()) })
                                .parse()
                        }
                    }
                }
            }
        }

        ItemFn {
            attrs: f.attrs,
            vis: Visibility::Inherited,

            sig: Signature {
                //                asyncness: None,
                inputs,
                ..f.sig.clone()
            },
            block: if true {
                f.block
            } else {
                Quote::new(sig.asyncness.unwrap().span())
                    .quote_with(smart_quote!(Vars { body: block }, {
                        {
                            rweb::rt::tokio::runtime::Builder::new()
                                .basic_scheduler()
                                .enable_all()
                                .build()
                                .unwrap()
                                .block_on(async { body })
                        }
                    }))
                    .parse()
            },
        }
    };

    //    let args: Punctuated<Ident, Token![,]> = sig
    //        .inputs
    //        .pairs()
    //        .enumerate()
    //        .map(|(i, pair)| {
    //            let (arg, comma) = pair.into_tuple();
    //
    //            Pair::new(Ident::new(&format!("arg{}", i), arg.span()),
    // comma.clone())        })
    //        .collect();

    let expr = if sig.asyncness.is_some() {
        q!(
            Vars {
                handler: &sig.ident,
                expr
            },
            { expr.and_then(handler) }
        )
    } else {
        q!(
            Vars {
                handler: &sig.ident,
                expr
            },
            { expr.map(handler) }
        )
    }
    .parse::<Expr>();

    q!(
        Vars {
            expr,
            handler: &sig.ident,
            Ret: match sig.output {
                ReturnType::Default => panic!("http handler should return type"),
                ReturnType::Type(_, ref ty) => ty,
            },
            handler_fn,
        },
        {
            #[allow(non_camel_case_types)]
            fn handler(
            ) -> impl rweb::Filter<Extract = (Ret,), Error = rweb::warp::Rejection>
                   + rweb::rt::Clone {
                use rweb::Filter;

                handler_fn

                expr
            }
        }
    )
    .into()
}

fn is_rweb_attr(a: &Attribute) -> bool {
    a.path.is_ident("json")
        || a.path.is_ident("form")
        || a.path.is_ident("body")
        || a.path.is_ident("query")
}
