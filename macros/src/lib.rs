extern crate proc_macro;
use pmutil::{q, Quote, ToTokensExt};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote::parse,
    punctuated::Punctuated,
    Attribute, Expr, FnArg, ItemFn, LitStr, Pat, Path, ReturnType, Signature, Token, Type,
    Visibility,
};

mod path;
mod route;
mod router;

#[proc_macro_attribute]
pub fn get(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ get })), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn post(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ post })), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn put(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ put })), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn delete(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ delete })), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn head(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ head })), path.into(), fn_item.into())
}

#[proc_macro_attribute]
pub fn options(
    path: proc_macro::TokenStream,
    fn_item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    compile_route(Some(q!({ options })), path.into(), fn_item.into())
}

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

fn compile_route(
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

    let (mut expr, vars) = path::compile(Some(expr), path, Some(sig), true);

    let handler_fn = {
        let mut inputs: Punctuated<FnArg, _> = f.sig.inputs.clone();
        let mut path_params = HashSet::new();

        {
            // Handle path parameters

            for (orig_idx, (name, idx)) in vars.into_iter().enumerate() {
                if path_params.contains(&orig_idx) {
                    continue;
                }

                match &f.sig.inputs[idx] {
                    FnArg::Typed(pat) => match *pat.pat {
                        Pat::Ident(ref i) if i.ident == name => {
                            inputs[orig_idx] = f.sig.inputs[idx].clone();
                            inputs[idx] = f.sig.inputs[orig_idx].clone();
                            path_params.insert(orig_idx);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        let inputs = {
            let mut actual_inputs = vec![];

            // Handle annotated parameters.
            for (idx, mut i) in inputs.into_pairs().enumerate() {
                if path_params.contains(&idx) {
                    actual_inputs.push(i);
                    continue;
                }

                let cloned_i = i.clone();

                match i.value_mut() {
                    FnArg::Receiver(_) => continue,
                    FnArg::Typed(ref mut pat) => {
                        if pat.attrs.is_empty() {
                            // If there's no attribute, it's type should implement FromRequest

                            actual_inputs.push(cloned_i);
                            expr = q!(Vars { expr, T: &pat.ty }, {
                                expr.and(<T as rweb::FromRequest>::new())
                            })
                            .parse();
                            continue;
                        }

                        let is_rweb_attr = pat.attrs.iter().any(is_rweb_arg_attr);
                        if !is_rweb_attr {
                            // We don't care about this parameter.
                            actual_inputs.push(i);
                            continue;
                        }

                        if pat.attrs.len() != 1 {
                            // TODO: Support cfg?
                            panic!("rweb currently support only one attribute on a parameter")
                        }

                        let attr = pat.attrs.iter().next().unwrap().clone();
                        pat.attrs = vec![];

                        if attr.path.is_ident("form") {
                            expr =
                                q!(Vars { expr }, { expr.and(rweb::filters::body::form()) }).parse()
                        } else if attr.path.is_ident("json") {
                            expr =
                                q!(Vars { expr }, { expr.and(rweb::filters::body::json()) }).parse()
                        } else if attr.path.is_ident("body") {
                            expr = q!(Vars { expr }, { expr.and(rweb::filters::body::bytes()) })
                                .parse()
                        } else if attr.path.is_ident("query") {
                            expr =
                                q!(Vars { expr }, { expr.and(rweb::filters::query::raw()) }).parse()
                        } else if attr.path.is_ident("cookie") {
                            if let Ok(cookie_name) = syn::parse2::<EqStr>(attr.tokens.clone()) {
                                expr = q!(
                                    Vars {
                                        expr,
                                        cookie_name: cookie_name.path
                                    },
                                    { expr.and(rweb::filters::cookie::cookie(cookie_name)) }
                                )
                                .parse();
                            } else {
                                panic!("#[cookie = \"foo\"] is used incorrectly")
                            }
                        } else if attr.path.is_ident("header") {
                            if let Ok(header_name) = syn::parse2::<EqStr>(attr.tokens.clone()) {
                                expr = q!(
                                    Vars {
                                        expr,
                                        header_name: header_name.path
                                    },
                                    { expr.and(rweb::filters::header::header(header_name)) }
                                )
                                .parse();
                            } else {
                                panic!(
                                    "invalid usage of header: {}\nCorrect usage is#[header = \
                                     \"accpet\"]",
                                    attr.tokens.dump()
                                )
                            }
                        } else if attr.path.is_ident("filter") {
                            let filter_path: EqStr = parse(attr.tokens.clone());
                            let filter_path = filter_path.path.value();
                            let tts: TokenStream = filter_path.parse().expect("failed tokenize");
                            let filter_path: Path = parse(tts);

                            expr =
                                q!(Vars { expr, filter_path }, { expr.and(filter_path()) }).parse();
                        } else if attr.path.is_ident("data") {
                            let ident = match &*pat.pat {
                                Pat::Ident(i) => &i.ident,
                                _ => unimplemented!("#[data] with complex pattern"),
                            };

                            expr = q!(Vars { expr, ident }, {
                                expr.and(rweb::rt::provider(ident))
                            })
                            .parse();

                            data_inputs.push(i.value().clone());
                        }

                        // Don't add unit type to argument list
                        match i.value() {
                            FnArg::Typed(pat) => match &*pat.ty {
                                Type::Tuple(tuple) if tuple.elems.is_empty() => continue,
                                _ => {}
                            },
                            _ => {}
                        }

                        actual_inputs.push(i);
                    }
                }
            }

            actual_inputs.into_iter().collect()
        };

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

    let expr = route::compile_fn_attrs(expr, &mut f.attrs, false);

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

    let expr = route::compile_fn_attrs(expr, &mut f.attrs, true);

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

fn is_rweb_arg_attr(a: &Attribute) -> bool {
    a.path.is_ident("json")
        || a.path.is_ident("form")
        || a.path.is_ident("body")
        || a.path.is_ident("query")
        || a.path.is_ident("cookie")
        || a.path.is_ident("header")
        || a.path.is_ident("filter")
        || a.path.is_ident("data")
}
