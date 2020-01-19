use proc_macro2::TokenStream;
use syn::{parse_quote::parse, ItemFn, ItemStruct};

pub fn router(attr: TokenStream, item: TokenStream) -> ItemFn {
    let item: ItemStruct = parse(item);
}
