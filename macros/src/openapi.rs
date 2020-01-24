//!
//!
//!
//! # Rules
//!
//!  - We abuse `Parameter.ref_path` to store type name.

use crate::{
    parse::{Delimited, KeyValue, Paren},
    path::find_ty,
    route::EqStr,
    util::ItemImplExt,
};
use pmutil::{q, Quote, ToTokensExt};
use proc_macro2::TokenStream;
use rweb_openapi::v3_0::{Location, ObjectOrReference, Operation, Parameter, Schema};
use std::borrow::Cow;
use syn::{
    export::ToTokens,
    parse2,
    punctuated::{Pair, Punctuated},
    Attribute, Block, Data, DeriveInput, Expr, Field, FieldValue, Fields, GenericParam, ItemImpl,
    Lit, Meta, NestedMeta, Signature, Stmt, Token, TraitBound, TraitBoundModifier, TypeParamBound,
};

pub fn derive_schema(mut input: DeriveInput) -> TokenStream {
    fn extract_doc(attrs: &mut Vec<Attribute>) -> String {
        let mut doc = None;
        let mut comments = String::new();

        attrs.retain(|attr| {
            if attr.path.is_ident("schema") {
                let v = match parse2::<Paren<KeyValue>>(attr.tokens.clone()) {
                    Ok(v) if v.inner.key.value() == "description" => v.inner.value.value(),
                    _ => return true,
                };
                doc = Some(v);
                return false;
            }

            if attr.path.is_ident("doc") {
                let v = match parse2::<EqStr>(attr.tokens.clone()) {
                    Ok(v) => v.value,
                    _ => return true,
                };

                if !comments.is_empty() {
                    comments.push(' ');
                }
                comments.push_str(&v.value());
            }

            true
        });

        match doc {
            Some(v) => v,
            None => comments,
        }
    }

    fn handle_field(f: &mut Field) -> Stmt {
        let i = f.ident.as_ref().unwrap();

        let desc = extract_doc(&mut f.attrs);
        q!(
            Vars {
                name: i,
                desc,
                Type: &f.ty
            },
            {
                map.insert(rweb::rt::Cow::Borrowed(stringify!(name)), {
                    {
                        #[allow(unused_mut)]
                        let mut s = <Type as rweb::openapi::Entity>::describe();
                        let description = desc;
                        if !description.is_empty() {
                            s.description = rweb::rt::Cow::Borrowed(description);
                        }
                        s
                    }
                });
            }
        )
        .parse()
    }

    fn handle_fields(fields: &mut Fields) -> Block {
        // Properties
        let mut block: Block = q!({ {} }).parse();
        block.stmts.push(
            q!({
                #[allow(unused_mut)]
                let mut map: rweb::rt::BTreeMap<rweb::rt::Cow<'static, str>, _> =
                    rweb::rt::BTreeMap::default();
            })
            .parse(),
        );

        for f in fields {
            block.stmts.push(handle_field(f));
        }

        block.stmts.push(Stmt::Expr(q!({ map }).parse()));

        block
    }

    let desc = extract_doc(&mut input.attrs);

    let mut component = None;
    input.attrs.retain(|attr| {
        if attr.path.is_ident("schema") {
            for config in parse2::<Paren<Delimited<Meta>>>(attr.tokens.clone())
                .expect("schema config is invalid")
                .inner
                .inner
            {
                match config {
                    Meta::Path(..) => unimplemented!("Meta::Path in #[schema]"),
                    Meta::NameValue(n) => {
                        //
                        if n.path.is_ident("component") {
                            assert!(
                                component.is_none(),
                                "duplicate #[schema(component = \"foo\")] detected"
                            );
                            component = Some(match n.lit {
                                Lit::Str(s) => s.value(),
                                l => panic!(
                                    "#[schema]: value of component should be a string literal, \
                                     but got {}",
                                    l.dump()
                                ),
                            })
                        } else {
                            panic!("#[schema]: Unknown option {}", n.path.dump())
                        }
                    }
                    Meta::List(l) => unimplemented!("Meta::List in #[schema]: {}", l.dump()),
                }
            }
        }

        true
    });

    let mut fields: Punctuated<FieldValue, Token![,]> = Default::default();

    match input.data {
        Data::Struct(ref mut data) => {
            {
                let block = handle_fields(&mut data.fields);
                fields.push(q!(Vars { block }, { properties: block }).parse());
            }

            fields.push(q!({ schema_type: rweb::openapi::Type::Object }).parse());
        }
        Data::Enum(ref mut data) => {
            let exprs: Punctuated<Expr, Token![,]> = data
                .variants
                .iter_mut()
                .filter_map(|v| {
                    let desc = extract_doc(&mut v.attrs);

                    match v.fields {
                        Fields::Named(ref f) if f.named.len() == 1 => None,

                        Fields::Named(..) => Some(Pair::Punctuated(
                            {
                                let fields = handle_fields(&mut v.fields);
                                q!(
                                    Vars { fields, desc },
                                    ({
                                        #[allow(unused_mut)]
                                        let mut s = rweb::openapi::Schema {
                                            fields,
                                            ..rweb::rt::Default::default()
                                        };
                                        let description = desc;
                                        if !description.is_empty() {
                                            s.description = rweb::rt::Cow::Borrowed(description);
                                        }

                                        rweb::openapi::ObjectOrReference::Object(s)
                                    })
                                )
                                .parse()
                            },
                            Default::default(),
                        )),
                        Fields::Unnamed(ref f) => {
                            //
                            assert!(f.unnamed.len() <= 1);
                            if f.unnamed.len() == 0 {
                                return None;
                            }

                            Some(Pair::Punctuated(
                                q!(
                                    Vars {
                                        Type: &f.unnamed.first().unwrap().ty,
                                        desc
                                    },
                                    ({
                                        #[allow(unused_mut)]
                                        let mut s = <Type as rweb::openapi::Entity>::describe();
                                        let description = desc;
                                        if !description.is_empty() {
                                            s.description = rweb::rt::Cow::Borrowed(description);
                                        }

                                        rweb::openapi::ObjectOrReference::Object(s)
                                    })
                                )
                                .parse(),
                                Default::default(),
                            ))
                        }
                        Fields::Unit => None,
                    }
                })
                .collect();

            fields.push(q!(Vars { exprs }, { one_of: vec![exprs] }).parse());
        }
        Data::Union(_) => unimplemented!("#[derive(Schema)] for union"),
    }

    let mut item = if let Some(comp) = component {
        let path_to_schema = format!("#/components/schemas/{}", comp);

        q!(
            Vars {
                Type: &input.ident,
                desc,
                path_to_schema,
                fields,
                comp,
            },
            {
                impl rweb::openapi::Entity for Type {
                    fn describe() -> rweb::openapi::Schema {
                        rweb::openapi::Schema {
                            ref_path: rweb::rt::Cow::Borrowed(path_to_schema),
                            ..rweb::rt::Default::default()
                        }
                    }

                    fn describe_component(
                    ) -> Option<(rweb::rt::Cow<'static, str>, rweb::openapi::Schema)>
                    {
                        Some((
                            rweb::rt::Cow::Borrowed(comp),
                            rweb::openapi::Schema {
                                fields,
                                description: rweb::rt::Cow::Borrowed(desc),
                                ..rweb::rt::Default::default()
                            },
                        ))
                    }
                }
            }
        )
    } else {
        q!(
            Vars {
                Type: &input.ident,
                desc,
                fields
            },
            {
                impl rweb::openapi::Entity for Type {
                    fn describe() -> rweb::openapi::Schema {
                        rweb::openapi::Schema {
                            fields,
                            description: rweb::rt::Cow::Borrowed(desc),
                            ..rweb::rt::Default::default()
                        }
                    }
                }
            }
        )
    }
    .parse::<ItemImpl>()
    .with_generics(input.generics.clone());

    for param in item.generics.params.iter_mut() {
        match param {
            GenericParam::Type(ref mut ty) => ty.bounds.push(TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: q!({ rweb::openapi::Entity }).parse(),
            })),
            _ => continue,
        }
    }

    item.dump()
}

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
    let mut has_description = false;

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
                } else if config.path().is_ident("description") {
                    match config {
                        Meta::NameValue(v) => match v.lit {
                            Lit::Str(s) => {
                                op.description = Cow::Owned(s.value());
                                has_description = true;
                            }
                            _ => panic!("#[openapi]: invalid operation summary"),
                        },
                        _ => panic!("Correct usage: #[openapi(summary = \"foo\")]"),
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

        if attr.path.is_ident("doc") && !has_description {
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
