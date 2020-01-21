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
    let mut data_inputs: Punctuated<_, Token![,]> = Default::default();

    let attr: Input = parse(attr);

    let (expr, path_vars) = crate::path::compile(None, attr.path.dump(), &f.sig, false);
    let (mut expr, inputs) =
        crate::route::param::compile(expr, &f.sig, &mut data_inputs, path_vars);

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
    let mut ret = q!(Vars { expr, router_name }, {
        #[allow(non_snake_case)]
        fn router_name(
        ) -> impl Clone + rweb::Filter<Extract = (impl rweb::Reply,), Error = rweb::Rejection>
        {
            use rweb::{rt::StatusCode, Filter};

            expr
        }
    })
    .parse::<ItemFn>();

    ret.sig.inputs = inputs;
    ret.vis = vis.clone();

    ret
}
