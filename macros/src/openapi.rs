use crate::{
    parse::{Delimited, Paren},
    path::find_ty,
};
use pmutil::{q, Quote, ToTokensExt};
use rweb_openapi::v3_0::{ObjectOrReference, Operation, Parameter};
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
        .map(|tag| Pair::Punctuated(q!(Vars { tag }, { tag.to_string() }), Default::default()))
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
                summary: summary_v.to_string(),
                description: description_v.to_string(),
                external_docs: Default::default(),
                operation_id: id_v.to_string(),
                parameters: vec![params_v],
                request_body: Default::default(),
                responses: Default::default(),
                callbacks: Default::default(),
                deprecated: Default::default(),
                servers: Default::default(),
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

    assert_eq!(
        param.schema, None,
        "Schema of a parameter should be provided using type system"
    );

    q!(
        Vars {
            name_v: &param.name,
            location_v: &param.location,
            required_v,
        },
        {
            rweb::openapi::ObjectOrReference::Object(rweb::openapi::Parameter {
                name: name_v.to_string(),
                location: location_v.to_string(),
                required: required_v,
                schema: None,
                unique_items: None,
                param_type: "".to_string(),
                format: "".to_string(),
                description: "".to_string(),
                style: None,
            })
        }
    )
    .parse()
}

pub fn parse(path: &str, sig: &Signature, attrs: &mut Vec<Attribute>) -> Operation {
    let mut op = Operation::default();

    for segment in path.split('/').filter(|&s| s != "") {
        if !segment.starts_with('{') {
            continue;
        }

        let var = &segment[1..segment.len() - 1];
        if let Some(_ty) = find_ty(sig, var) {
            let mut p = Parameter::default();
            p.name = var.to_string();
            p.location = "path".to_string();
            p.required = Some(true);

            op.parameters.push(ObjectOrReference::Object(Parameter {
                name: var.to_string(),
                location: "path".to_string(),
                required: Some(true),
                schema: None,
                unique_items: None,
                param_type: "".to_string(),
                format: "".to_string(),
                description: "".to_string(),
                style: None,
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
                            Lit::Str(s) => op.operation_id = s.value(),
                            _ => panic!("#[openapi]: invalid operation id"),
                        },
                        _ => panic!("Correct usage: #[openapi(id = \"foo\")]"),
                    }
                } else if config.path().is_ident("summary") {
                    match config {
                        Meta::NameValue(v) => match v.lit {
                            Lit::Str(s) => op.summary = s.value(),
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
                                        Lit::Str(s) => op.tags.push(s.value()),
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

        if attr.path.is_ident("doc") {}

        true
    });

    op
}
