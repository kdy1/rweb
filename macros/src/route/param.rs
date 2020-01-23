use crate::route::EqStr;
use pmutil::{q, ToTokensExt};
use proc_macro2::TokenStream;
use std::collections::HashSet;
use syn::{
    parse_quote::parse, punctuated::Punctuated, Attribute, Expr, FnArg, Pat, Path, Signature,
    Token, Type,
};

/// Returns (expr, actual_inputs_of_handler, from_request_types)
pub fn compile(
    mut expr: Expr,
    sig: &Signature,
    data_inputs: &mut Punctuated<FnArg, Token![,]>,
    path_vars: Vec<(String, usize)>,
    insert_data_provider: bool,
) -> (Expr, Punctuated<FnArg, Token![,]>, Vec<Type>) {
    let mut path_params = HashSet::new();
    let mut inputs = sig.inputs.clone();
    let mut from_request_types = vec![];

    {
        // Handle path parameters

        for (orig_idx, (name, idx)) in path_vars.into_iter().enumerate() {
            if path_params.contains(&orig_idx) {
                continue;
            }

            match &sig.inputs[idx] {
                FnArg::Typed(pat) => match *pat.pat {
                    Pat::Ident(ref i) if i.ident == name => {
                        inputs[orig_idx] = sig.inputs[idx].clone();
                        inputs[idx] = sig.inputs[orig_idx].clone();
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
                        from_request_types.push(*pat.ty.clone());
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
                        expr = q!(Vars { expr }, { expr.and(rweb::filters::body::form()) }).parse()
                    } else if attr.path.is_ident("json") {
                        expr = q!(Vars { expr }, { expr.and(rweb::filters::body::json()) }).parse()
                    } else if attr.path.is_ident("body") {
                        expr = q!(Vars { expr }, { expr.and(rweb::filters::body::bytes()) }).parse()
                    } else if attr.path.is_ident("query") {
                        expr = q!(Vars { expr }, { expr.and(rweb::filters::query::raw()) }).parse()
                    } else if attr.path.is_ident("cookie") {
                        if let Ok(cookie_name) = syn::parse2::<EqStr>(attr.tokens.clone()) {
                            expr = q!(
                                Vars {
                                    expr,
                                    cookie_name: cookie_name.value
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
                                    header_name: header_name.value
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
                        let filter_path = filter_path.value.value();
                        let tts: TokenStream = filter_path.parse().expect("failed tokenize");
                        let filter_path: Path = parse(tts);

                        expr = q!(Vars { expr, filter_path }, { expr.and(filter_path()) }).parse();
                    } else if attr.path.is_ident("data") {
                        let ident = match &*pat.pat {
                            Pat::Ident(i) => &i.ident,
                            _ => unimplemented!("#[data] with complex pattern"),
                        };

                        if insert_data_provider {
                            expr = q!(Vars { expr, ident }, {
                                expr.and(rweb::rt::provider(ident))
                            })
                            .parse();
                        }

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

    (expr, inputs, from_request_types)
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
