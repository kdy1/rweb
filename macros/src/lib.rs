extern crate proc_macro;

use pmutil::{q, Quote, ToTokensExt};
use proc_macro2::TokenStream;
use syn::{parse_quote::parse, Item, ItemFn};

#[proc_macro_attribute]
pub fn get(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "get" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn post(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "post" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn put(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "put" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn delete(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "delete" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn head(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "head" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn connect(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "connect" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn options(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "options" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn trace(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "trace" }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn patch(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ "patch" }), path.into(), fn_item.into())
}

fn expand(method: Quote, path: TokenStream, fn_item: TokenStream) -> proc_macro::TokenStream {
    let fn_item: ItemFn = parse(fn_item);
    let sig = fn_item.sig;

    q!(Vars { Item: &sig.ident }, {
        struct Item;

        impl rweb::HttpServiceFactory for Item {}
    })
    .into()
}
