extern crate proc_macro;

use pmutil::{q, Quote};
use proc_macro2::TokenStream;
use syn::{parse_quote::parse, ItemFn, ReturnType};

#[proc_macro_attribute]
pub fn get(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ get }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn post(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ post }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn put(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ put }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn delete(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ delete }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn head(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ head }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn connect(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ connect }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn options(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ options }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn trace(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ trace }), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn patch(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    expand(q!({ patch }), path.into(), fn_item.into())
}

fn expand(method: Quote, path: TokenStream, fn_item: TokenStream) -> proc_macro::TokenStream {
    let fn_item: ItemFn = parse(fn_item);
    let sig = &fn_item.sig;

    q!(
        Vars {
            http_method: method,
            http_path: &path,
            Ret: match sig.output {
                ReturnType::Default => q!({ () }),
                ReturnType::Type(_, ref ty) => q!(Vars { ty }, { ty }),
            },
            Item: &sig.ident,
            body: &fn_item.block
        },
        {
            #[allow(non_camel_case_types)]
            struct Item;

            impl rweb::service::HttpServiceFactory for Item {
                fn register(self, config: &mut rweb::service::Registry) {
                    async fn Item() -> Ret {
                        body
                    }

                    let resource = rweb::Resource::new(http_path)
                        .name(stringify!(Item))
                        .guard(rweb::guard::http::http_method())
                        .to(Item);
                    rweb::service::HttpServiceFactory::register(resource, config)
                }
            }
        }
    )
    .into()
}
