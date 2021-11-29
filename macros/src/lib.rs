//! proc macros for rweb

#![deny(missing_docs)]

extern crate proc_macro;
use self::route::compile_route;
use pmutil::{q, ToTokensExt};

mod openapi;
mod parse;
mod path;
mod route;
mod router;
mod util;

/// Creates a get method route handler
#[proc_macro_attribute]
pub fn get(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ get })), path.into(), fn_item.into())
}

/// Creates a post method route handler
#[proc_macro_attribute]
pub fn post(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ post })), path.into(), fn_item.into())
}

/// Creates a put method route handler
#[proc_macro_attribute]
pub fn put(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ put })), path.into(), fn_item.into())
}

/// Creates a delete method route handler
#[proc_macro_attribute]
pub fn delete(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ delete })), path.into(), fn_item.into())
}

/// Creates a head method route handler
#[proc_macro_attribute]
pub fn head(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ head })), path.into(), fn_item.into())
}

/// Creates a options method route handler
#[proc_macro_attribute]
pub fn options(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ options })), path.into(), fn_item.into())
}

/// Creates a patch method route handler
#[proc_macro_attribute]
pub fn patch(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ patch })), path.into(), fn_item.into())
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

/// Implements Entity for the type.
///
/// See documentation of Entity for details and examples.
#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive_schema(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !cfg!(feature = "openapi") {
        return "".parse().unwrap();
    }
    let input = syn::parse::<syn::DeriveInput>(input).expect("failed to parse derive input");
    openapi::derive_schema(input).into()
}
