//! A macro to convert a function to rweb handler.

extern crate proc_macro;

use pmutil::{q, Quote};
use proc_macro2::TokenStream;
use syn::{parse_quote::parse, FnArg, ItemFn, Pat, Signature, Visibility};

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

    let (path, map) = path::compile(path, sig);

    let handler_fn = {
        let mut sig = f.sig.clone();

        for (orig_idx, (name, idx)) in map.into_iter().enumerate() {
            match &f.sig.inputs[idx] {
                FnArg::Typed(pat) => match *pat.pat {
                    Pat::Ident(ref i) if i.ident == name && orig_idx != idx => {
                        sig.inputs[orig_idx] = f.sig.inputs[idx].clone();
                        sig.inputs[idx] = f.sig.inputs[orig_idx].clone();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        ItemFn {
            attrs: f.attrs,
            vis: Visibility::Inherited,
            sig: Signature {
                asyncness: None,
                ..sig
            },
            block: f.block,
        }
    };

    q!(
        Vars {
            http_method: method,
            http_path: &path,
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

                http_path
                    .and(rweb::filters::method::http_method())
                    .map(handler)
            }
        }
    )
    .into()
}
