extern crate proc_macro;

use pmutil::{q, Quote};
use proc_macro2::TokenStream;
use std::collections::HashMap;
use syn::{parse_quote::parse, Expr, ItemFn, LitStr, ReturnType};

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

fn expand_route(method: Quote, path: TokenStream, fn_item: TokenStream) -> proc_macro::TokenStream {
    let fn_item: ItemFn = parse(fn_item);
    let sig = &fn_item.sig;

    let (path, _) = expand_path(path);

    q!(
        Vars {
            http_method: method,
            http_path: &path,
            Ret: match sig.output {
                ReturnType::Default => q!({ () }),
                ReturnType::Type(_, ref ty) => q!(Vars { ty }, { ty }),
            },
            handler: &sig.ident,
            body: &fn_item.block
        },
        {
            #[allow(non_camel_case_types)]
            fn handler(
            ) -> impl rweb::Filter<Extract = impl rweb::reply::Reply, Error = rweb::warp::Rejection>
                   + ::std::clone::Clone {
                use rweb::Filter;

                fn handler() -> Ret {
                    body
                }

                http_path
                    .and(rweb::filters::method::http_method())
                    .map(handler)
            }
        }
    )
    .into()
}

fn expand_path(path: TokenStream) -> (Expr, HashMap<String, String>) {
    let http_path: LitStr = parse(path);

    let expr = q!(Vars { http_path }, { rweb::filters::path::path(http_path) }).parse();

    (expr, Default::default())
}
