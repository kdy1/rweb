use pmutil::{q, Quote, ToTokensExt};
use proc_macro2::TokenStream;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote::parse,
    punctuated::Punctuated,
    Expr, ItemFn, LitStr, ReturnType, Signature, Token, Visibility,
};

pub mod fn_attr;
pub mod param;

/// An eq token followed by literal string
struct EqStr {
    _eq: Token![=],
    path: LitStr,
}

impl Parse for EqStr {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        Ok(EqStr {
            _eq: input.parse()?,
            path: input.parse()?,
        })
    }
}

/// An eq token followed by literal string
struct ParenTwoValue {
    key: LitStr,
    _eq: Token![,],
    value: LitStr,
}

impl Parse for ParenTwoValue {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let content;
        parenthesized!(content in input);
        Ok(ParenTwoValue {
            key: content.parse()?,
            _eq: content.parse()?,
            value: content.parse()?,
        })
    }
}

pub fn compile_route(
    method: Option<Quote>,
    path: TokenStream,
    f: TokenStream,
) -> proc_macro::TokenStream {
    let mut f: ItemFn = parse(f);
    let sig = &f.sig;
    let mut data_inputs: Punctuated<_, Token![,]> = Default::default();

    // Apply method filter
    let expr: Expr = if let Some(method) = method {
        q!(
            Vars {
                http_method: method,
            },
            { rweb::filters::method::http_method() }
        )
        .parse()
    } else {
        q!({ rweb::filters::any() }).parse()
    };

    let (mut expr, vars) = crate::path::compile(Some(expr), path, Some(sig), true);

    let handler_fn = {
        let (e, inputs) = param::compile(expr, &f.sig, &mut data_inputs, vars, true);
        expr = e;
        ItemFn {
            attrs: Default::default(),
            vis: Visibility::Inherited,

            sig: Signature {
                //                asyncness: None,
                inputs,
                ..f.sig.clone()
            },
            block: f.block,
        }
    };

    let should_use_impl_trait =
        sig.asyncness.is_some() || f.attrs.iter().any(|attr| attr.path.is_ident("cors"));

    let expr = fn_attr::compile_fn_attrs(expr, &mut f.attrs, false);

    let expr = if sig.asyncness.is_some() {
        q!(
            Vars {
                handler: &sig.ident,
                expr
            },
            { expr.and_then(handler) }
        )
    } else {
        q!(
            Vars {
                handler: &sig.ident,
                expr
            },
            { expr.map(handler) }
        )
    }
    .parse::<Expr>();

    let expr = fn_attr::compile_fn_attrs(expr, &mut f.attrs, true);

    let ret = if should_use_impl_trait {
        q!((impl rweb::Reply)).dump()
    } else {
        match sig.output {
            ReturnType::Default => panic!("http handler should return type"),
            ReturnType::Type(_, ref ty) => ty.dump(),
        }
    };

    let mut outer = q!(
        Vars {
            expr,
            handler: &sig.ident,
            Ret: ret,
            handler_fn,
        },
        {
            fn handler(
            ) -> impl rweb::Filter<Extract = (Ret,), Error = rweb::warp::Rejection>
                   + rweb::rt::Clone {
                use rweb::Filter;

                handler_fn

                expr
            }
        }
    )
    .parse::<ItemFn>();

    outer.vis = f.vis;
    outer.sig = Signature {
        inputs: data_inputs,
        ..outer.sig
    };

    outer.dump().into()
}
