//! A macro to convert a function to rweb handler.
//!
//! # Attribute on parameters
//!
//! ## `#[body]`
//! Parses request body.

extern crate proc_macro;

use pmutil::{q, Quote};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::{
    parse_quote::parse, punctuated::Punctuated, Attribute, Expr, FnArg, ItemFn, Pat, Signature,
    Visibility,
};

mod path;

#[proc_macro_attribute]
pub fn get(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_route(q!({ get }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn post(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_route(q!({ post }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn put(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_route(q!({ put }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn delete(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_route(q!({ delete }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn head(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_route(q!({ head }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn options(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_route(q!({ options }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn patch(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand_route(q!({ patch }), path.into(), fn_item.into())
}

fn expand_route(method: Quote, path: TokenStream, f: TokenStream) -> proc_macro::TokenStream {
    let f: ItemFn = parse(f);
    let sig = &f.sig;

    // Apply method filter
    let expr: Expr = q!(
        Vars {
            http_method: method,
        },
        { rweb::filters::method::http_method() }
    )
    .parse();

    let (mut expr, vars) = path::compile(expr, path, sig);

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
                inputs,
                ..f.sig.clone()
            },
            block: f.block,
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
                   + ::std::clone::Clone {
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
