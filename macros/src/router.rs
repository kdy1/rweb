use crate::path::compile;
use pmutil::{q, ToTokensExt};
use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote::parse,
    punctuated::Punctuated,
    Error, Expr, ItemFn, LitStr, Meta, Token,
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
    let f: ItemFn = parse(item);
    let router_name = &f.sig.ident;
    let vis = &f.vis;

    let attr: Input = parse(attr);

    let (mut expr, _) = compile(None, attr.path.dump(), None, false);
    let mut exprs: Punctuated<Expr, Token![.]> = Punctuated::default();

    match attr.services {
        Meta::List(list) => {
            if list.path.is_ident("services") {
                for name in list.nested.into_iter() {
                    if exprs.is_empty() {
                        exprs.push(q!(Vars { name }, { name() }).parse());
                    } else {
                        exprs.push(q!(Vars { name }, { or(name()) }).parse());
                    }
                }

                expr = q!(Vars { exprs, expr }, { expr.and(exprs) }).parse();
            } else {
                panic!("Unknown path {}", list.path.dump())
            }
        }

        _ => panic!("#[router(\"/path\", services(a, b, c,))] is correct usage"),
    }

    // TODO: Default handler
    q!(
        Vars {
            vis,
            expr,
            router_name
        },
        {
            #[allow(non_snake_case)]
            vis fn router_name(
            ) -> impl Clone + rweb::Filter<Extract = (impl rweb::Reply,), Error = rweb::Rejection>
            {
                use rweb::{rt::StatusCode, Filter};

                expr
            }
        }
    )
    .parse()
}
