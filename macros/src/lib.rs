//! A macro to convert a function to rweb handler.
//!
//! # Attribute on parameters
//!
//! ## `#[body]`
//! Parses request body.

extern crate proc_macro;
use pmutil::{q, smart_quote, Quote, ToTokensExt};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::{
    parse_quote::parse, punctuated::Punctuated, spanned::Spanned, Attribute, Expr, FnArg, ItemFn,
    Pat, Signature, Visibility,
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
                        }
                    }
                }
            }
        }

        ItemFn {
            attrs: f.attrs,
            vis: Visibility::Inherited,
            sig: Signature {
                asyncness: None,
                inputs,
                ..f.sig.clone()
            },
            block: if sig.asyncness.is_none() {
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

    q!(
        Vars {
            expr,
            handler: &sig.ident,
            handler_fn,
        },
        {
            #[allow(non_camel_case_types)]
            fn handler(
            ) -> impl rweb::Filter<Extract = impl rweb::reply::Reply, Error = rweb::warp::Rejection>
                   + rweb::rt::Clone {
                use rweb::Filter;

                handler_fn

                expr.map(handler)
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
