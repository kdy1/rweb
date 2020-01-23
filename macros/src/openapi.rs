//!
//!
//!
//! # Rules
//!
//!  - We abuse `Parameter.ref_path` to store type name.

use crate::{
    parse::{Delimited, Paren},
    path::find_ty,
    route::EqStr,
};
use pmutil::{q, Quote, ToTokensExt};
use proc_macro2::TokenStream;
use rweb_openapi::v3_0::{Location, ObjectOrReference, Operation, Parameter, Schema};
use std::borrow::Cow;
use syn::{
    export::ToTokens,
    parse2,
    punctuated::{Pair, Punctuated},
    Attribute, Expr, Lit, Meta, NestedMeta, Signature, Token,
};

pub fn quote_op(op: Operation) -> Expr {
    let tags_v: Punctuated<Quote, Token![,]> = op
        .tags
        .iter()
        .map(|tag| {
            Pair::Punctuated(
                q!(Vars { tag }, { rweb::rt::Cow::Borrowed(tag) }),
                Default::default(),
            )
        })
        .collect();

    let params_v: Punctuated<Expr, Token![,]> = op
        .parameters
        .iter()
        .map(|v| Pair::Punctuated(quote_parameter(v), Default::default()))
        .collect();

    q!(
        Vars {
            tags_v,
            id_v: op.operation_id,
            summary_v: op.summary,
            description_v: op.description,
            params_v,
        },
        {
            rweb::openapi::Operation {
                tags: vec![tags_v],
                summary: rweb::rt::Cow::Borrowed(summary_v),
                description: rweb::rt::Cow::Borrowed(description_v),
                operation_id: rweb::rt::Cow::Borrowed(id_v),
                parameters: vec![params_v],
                ..Default::default()
            }
        }
    )
    .parse()
}

/// TODO: Move this to pmutil
fn quote_option<T>(o: Option<T>) -> Quote
where
    T: ToTokens,
{
    match o {
        Some(v) => q!(Vars { v }, { Some(v) }),
        None => q!({ None }),
    }
}

fn quote_parameter(param: &ObjectOrReference<Parameter>) -> Expr {
    let param = match param {
        ObjectOrReference::Ref { .. } => unreachable!("quote_parameter(ObjectOrReference::Ref)"),
        ObjectOrReference::Object(param) => param,
    };

    let required_v = quote_option(param.required);

    assert!(
        param.schema.is_some(),
        "Schema should contain a (rust) path to the type"
    );
    let ty = param
        .schema
        .as_ref()
        .unwrap()
        .ref_path
        .parse::<TokenStream>()
        .expect("failed to lex path to the type of parameter?");

    q!(
        Vars {
            Type: ty,
            name_v: &param.name,
            location_v: quote_location(param.location),
            required_v,
        },
        {
            rweb::openapi::ObjectOrReference::Object(rweb::openapi::Parameter {
                name: rweb::rt::Cow::Borrowed(name_v),
                location: location_v,
                required: required_v,
                schema: Some(<Type as rweb::openapi::Entity>::describe()),
                ..Default::default()
            })
        }
    )
    .parse()
}

fn quote_location(l: Location) -> Quote {
    match l {
        Location::Query => q!({ rweb::openapi::Location::Query }),
        Location::Header => q!({ rweb::openapi::Location::Header }),
        Location::Path => q!({ rweb::openapi::Location::Path }),
        Location::FormData => q!({ rweb::openapi::Location::FormData }),
    }
}

pub fn parse(path: &str, sig: &Signature, attrs: &mut Vec<Attribute>) -> Operation {
    let mut op = Operation::default();

    for segment in path.split('/').filter(|&s| s != "") {
        if !segment.starts_with('{') {
            continue;
        }

        let var = &segment[1..segment.len() - 1];
        if let Some(ty) = find_ty(sig, var) {
            let mut p = Parameter::default();
            p.name = Cow::Owned(var.to_string());
            p.location = Location::Path;
            p.required = Some(true);

            op.parameters.push(ObjectOrReference::Object(Parameter {
                name: Cow::Owned(var.to_string()),
                location: Location::Path,
                required: Some(true),
                schema: Some(Schema {
                    ref_path: Cow::Owned(ty.dump().to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }));
        }
    }

    attrs.retain(|attr| {
        if attr.path.is_ident("openapi") {
            // tags("foo", "bar", "baz)

            let configs = parse2::<Paren<Delimited<Meta>>>(attr.tokens.clone())
                .expect("openapi config is invalid")
                .inner
                .inner;

            for config in configs {
                if config.path().is_ident("id") {
                    assert!(
                        op.operation_id.is_empty(),
                        "#[openapi]: Duplicate operation id detected"
                    );
                    match config {
                        Meta::NameValue(v) => match v.lit {
                            Lit::Str(s) => op.operation_id = Cow::Owned(s.value()),
                            _ => panic!("#[openapi]: invalid operation id"),
                        },
                        _ => panic!("Correct usage: #[openapi(id = \"foo\")]"),
                    }
                } else if config.path().is_ident("summary") {
                    match config {
                        Meta::NameValue(v) => match v.lit {
                            Lit::Str(s) => op.summary = Cow::Owned(s.value()),
                            _ => panic!("#[openapi]: invalid operation summary"),
                        },
                        _ => panic!("Correct usage: #[openapi(summary = \"foo\")]"),
                    }
                } else if config.path().is_ident("tags") {
                    match config {
                        Meta::List(l) => {
                            for tag in l.nested {
                                match tag {
                                    NestedMeta::Lit(v) => match v {
                                        Lit::Str(s) => op.tags.push(Cow::Owned(s.value())),
                                        _ => panic!("#[openapi]: tag should be a string literal"),
                                    },
                                    _ => panic!("Correct usage: #[openapi(tags(\"foo\" ,\"bar\")]"),
                                }
                            }
                        }
                        _ => panic!("Correct usage: #[openapi(tags(\"foo\" ,\"bar\")]"),
                    }
                } else {
                    panic!("Unknown openapi config `{}`", config.dump())
                }
            }

            return false;
        }

        if attr.path.is_ident("doc") {
            let s: EqStr = parse2(attr.tokens.clone()).expect("failed to parse comments");
            if !op.description.is_empty() {
                op.description.to_mut().push(' ');
            }
            op.description
                .to_mut()
                .push_str(&s.value.value().trim_start());
            // Preserve comments
            return true;
        }

        true
    });

    op
}
