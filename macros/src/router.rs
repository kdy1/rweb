use crate::path::compile;
use pmutil::{q, ToTokensExt};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote::parse,
    Error, ItemFn, ItemStruct, LitStr, Meta, MetaList, Signature, Token,
};

struct Input {
    path: LitStr,
    _comma: Token![,],
    services: Meta,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> Result<Self, Error> {
        Ok(Input {
            path: input.parse()?,
            _comma: input.parse()?,
            services: input.parse()?,
        })
    }
}

pub fn router(attr: TokenStream, item: TokenStream) -> ItemFn {
    let item: ItemStruct = parse(item);
    let router_name = &item.ident;

    if !item.fields.is_empty() {
        panic!("#[router] should be applied to unit struct")
    }

    let attr: Input = parse(attr);

    match attr.services {
        Meta::List(list) => {
            if list.path.is_ident("services") {
            } else {
                panic!("Unknown path {}", list.path.dump())
            }
        }

        _ => panic!("#[router(\"/path\", services(a, b, c,))] is correct usage"),
    }

    let expr = compile(None, attr.path.dump(), None, false);

    q!(Vars { router_name }, {
        fn router_name(
        ) -> impl Clone + rweb::Filter<Extract = impl rweb::Reply, Error = rweb::Rejection>
        {
            use rweb::Filter;

            expr
        }
    })
    .parse()
}
