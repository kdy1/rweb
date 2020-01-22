use crate::route::fn_attr::compile_fn_attrs;
use pmutil::{q, ToTokensExt};
use proc_macro2::{Ident, TokenStream};
use syn::{
    parse::{Parse, ParseStream},
    parse2,
    punctuated::{Pair, Punctuated},
    Error, Expr, FnArg, ItemFn, LitStr, Meta, Pat, Token,
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
    let mut f: ItemFn = parse2(item).expect("failed to parse input as a function item");
    let router_name = &f.sig.ident;
    let vis = &f.vis;
    let mut data_inputs: Punctuated<_, Token![,]> = Default::default();

    let attr: Input = parse2(attr).expect("failed to parse input as Input { path , service }");

    let (expr, path_vars) = crate::path::compile(None, attr.path.dump(), None, false);
    let (expr, inputs) =
        crate::route::param::compile(expr, &f.sig, &mut data_inputs, path_vars, false);

    let mut exprs: Punctuated<Expr, Token![.]> = Punctuated::default();

    let args: Punctuated<Ident, _> = data_inputs
        .pairs()
        .map(|pair| {
            let p = pair.punct().cloned();
            let t = pair.value();

            let t = match t {
                FnArg::Typed(pat) => match &*pat.pat {
                    Pat::Ident(p) => p.ident.clone(),
                    _ => unimplemented!("proper error reporting for non-ident #[data] input"),
                },
                _ => unimplemented!("proper error reporting for non-ident #[data] input"),
            };
            //
            Pair::new(t, p)
        })
        .collect();

    let mut expr = compile_fn_attrs(expr, &mut f.attrs, false);

    match attr.services {
        Meta::List(ref list) => {
            if list.path.is_ident("services") {
                for name in list.nested.iter() {
                    if exprs.is_empty() {
                        exprs.push(q!(Vars { name, args: &args }, { name(args) }).parse());
                    } else {
                        exprs.push(q!(Vars { name, args: &args }, { or(name(args)) }).parse());
                    }
                }

                expr = q!(Vars { exprs, expr }, { expr.and(exprs) }).parse();
            } else {
                panic!("Unknown path {}", list.path.dump())
            }
        }

        _ => panic!("#[router(\"/path\", services(a, b, c,))] is correct usage"),
    }

    let expr = compile_fn_attrs(expr, &mut f.attrs, true);

    // TODO: Default handler
    let mut ret = q!(
        Vars {
            path: &attr.path,
            expr,
            router_name
        },
        {
            fn router_name(
            ) -> impl Clone + rweb::Filter<Extract = (impl rweb::Reply,), Error = rweb::Rejection>
            {
                use rweb::{rt::StatusCode, Filter};

                rweb::openapi::with(|__collector: Option<&mut rweb::openapi::Collector>| {
                    if let Some(__collector) = __collector {
                        __collector.with_appended_prefix(stringify!(path), || expr)
                    } else {
                        expr
                    }
                })
            }
        }
    )
    .parse::<ItemFn>();

    ret.attrs = f.attrs;
    ret.sig.inputs = inputs;
    ret.vis = vis.clone();

    ret
}
