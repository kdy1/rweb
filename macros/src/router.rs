use crate::path::compile;
use pmutil::{q, ToTokensExt};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote::parse,
    Error, ItemFn, ItemStruct, LitStr, Meta, Token,
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

    let (mut expr, _) = compile(None, attr.path.dump(), None, false);

    match attr.services {
        Meta::List(list) => {
            if list.path.is_ident("services") {
                for name in list.nested.into_iter() {
                    expr = q!(Vars { name, expr }, { expr.or(name()) }).parse();
                }
            } else {
                panic!("Unknown path {}", list.path.dump())
            }
        }

        _ => panic!("#[router(\"/path\", services(a, b, c,))] is correct usage"),
    }

    // TODO: Default handler
    q!(Vars { expr, router_name }, {
        #[allow(non_snake_case)]
        fn router_name(
        ) -> impl Clone + rweb::Filter<Extract = (impl rweb::Reply,), Error = rweb::Rejection>
        {
            use rweb::{rt::StatusCode, Filter};

            expr.and(rweb::filters::path::end())
                .map(|_| StatusCode::from_u16(404).unwrap())
        }
    })
    .parse()
}
