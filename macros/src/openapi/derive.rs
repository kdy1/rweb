use crate::{
    openapi::case::RenameRule,
    parse::{Delimited, KeyValue, Paren},
    route::EqStr,
    util::ItemImplExt,
};
use pmutil::{q, ToTokensExt};
use proc_macro2::{Ident, TokenStream};
use syn::{
    parse2,
    punctuated::{Pair, Punctuated},
    Attribute, Block, Data, DeriveInput, Expr, Field, FieldValue, Fields, GenericParam, ItemImpl,
    Lit, LitStr, Meta, MetaList, MetaNameValue, NestedMeta, Stmt, Token, TraitBound,
    TraitBoundModifier, TypeParamBound,
};

/// Extracts all `#[serde(..)]` attributes and flattens `Meta::List`s.
fn get_serde_meta_attrs<'a>(attrs: &'a [Attribute]) -> impl Iterator<Item = Meta> + 'a {
    attrs
        .iter()
        .filter_map(|attr| {
            if attr.path.is_ident("serde") {
                attr.parse_meta().ok()
            } else {
                None
            }
        })
        .flat_map(|meta| match meta {
            Meta::List(MetaList { nested, .. }) => nested
                .into_iter()
                .filter_map(|m| match m {
                    NestedMeta::Meta(m) => Some(m),
                    _ => None,
                })
                .collect(),
            m => vec![m],
        })
}

/// Search for `#[serde(rename_all = '')]`
fn get_rename_all(attrs: &[Attribute]) -> RenameRule {
    get_serde_meta_attrs(attrs)
        .find_map(|attr| match attr {
            Meta::NameValue(MetaNameValue {
                path,
                lit: Lit::Str(value),
                ..
            }) if path.is_ident("rename_all") => Some(value.value().parse().unwrap()),
            _ => None,
        })
        .unwrap_or(RenameRule::None)
}

/// Search for `#[serde(rename = '')]`
fn get_rename(attrs: &[Attribute]) -> Option<String> {
    get_serde_meta_attrs(attrs).find_map(|attr| match attr {
        Meta::NameValue(MetaNameValue {
            path,
            lit: Lit::Str(value),
            ..
        }) if path.is_ident("rename") => Some(value.value()),
        _ => None,
    })
}

/// Styles of representing an enum.
///
/// Copied from https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs
enum EnumTagType {
    /// The default.
    ///
    /// ```json
    /// {"variant1": {"key1": "value1", "key2": "value2"}}
    /// ```
    External,

    /// `#[serde(tag = "type")]`
    ///
    /// ```json
    /// {"type": "variant1", "key1": "value1", "key2": "value2"}
    /// ```
    Internal { tag: String },

    /// `#[serde(tag = "t", content = "c")]`
    ///
    /// ```json
    /// {"t": "variant1", "c": {"key1": "value1", "key2": "value2"}}
    /// ```
    Adjacent { tag: String, content: String },

    /// `#[serde(untagged)]`
    ///
    /// ```json
    /// {"key1": "value1", "key2": "value2"}
    /// ```
    None,
}

/// Search for `#[serde(tag = '')]`, `#[serde(content = '')]`, and `#[serde(untagged)]`.
fn get_enum_tag_type(attrs: &[Attribute]) -> EnumTagType {
    let mut untagged = false;
    let mut tag = None;
    let mut content = None;
    // Don't panic on invalid serde tags since serde will report them properly itself
    for attr in get_serde_meta_attrs(attrs) {
        match attr {
            Meta::Path(path) => {
                if path.is_ident("untagged") {
                    untagged = true;
                }
            }
            Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                if path.is_ident("tag") {
                    if let Lit::Str(s) = lit {
                        tag = Some(s.value());
                    }
                } else if path.is_ident("content") {
                    if let Lit::Str(s) = lit {
                        content = Some(s.value());
                    }
                }
            }
            _ => {}
        }
    }
    match (untagged, tag, content) {
        (false, None, None) => EnumTagType::External,
        (false, Some(tag), None) => EnumTagType::Internal { tag },
        (false, Some(tag), Some(content)) => EnumTagType::Adjacent { tag, content },
        (true, None, None) => EnumTagType::None,
        (untagged, tag, content) => panic!(
            "Invalid serde enum tag type configuration: untagged={}, tag={:?}, content={:?}",
            untagged, tag, content
        ),
    }
}

fn get_skip_mode(attrs: &[Attribute]) -> (bool, bool) {
    let mut ser = false;
    let mut de = false;
    for attr in get_serde_meta_attrs(attrs) {
        match attr {
            Meta::Path(pa) => {
                if pa.is_ident("skip") {
                    return (true, true);
                } else if pa.is_ident("skip_serializing") {
                    ser = true
                } else if pa.is_ident("skip_deserializing") {
                    de = true
                }
            }
            _ => {}
        }
    }
    (ser, de)
}

fn field_name(type_attrs: &[Attribute], field: &Field) -> String {
    if let Some(s) = get_rename(&field.attrs) {
        return s;
    }

    let rule = get_rename_all(type_attrs);

    rule.apply_to_field(&field.ident.as_ref().unwrap().to_string())
}

macro_rules! invalid_schema_usage {
    ($act:expr) => {
        // rust-lang/rust#54140
        // panic!("{}", $act.__span().error("Correct usage: #[schema(description =
        // \"foo\", example = \"bar\")]"));
        panic!(
            "Invalid schema usage: {}
Correct usage: #[schema(description = \"foo\", example = \"bar\")]",
            $act.dump()
        );
    };
}

fn extract_example(attrs: &Vec<Attribute>) -> Option<TokenStream> {
    let mut v = None;

    let mut process_nv = |n: syn::MetaNameValue| {
        if n.path.is_ident("example") {
            assert!(
                v.is_none(),
                "duplicate #[schema(example = \"foo\")] detected"
            );

            v = Some(match n.lit {
                Lit::Str(s) => s
                    .value()
                    .parse::<TokenStream>()
                    .expect("expected example to be path"),
                l => panic!(
                    "#[schema(example = \"foo\")]: value of example should be a string literal, \
                     but got {}",
                    l.dump()
                ),
            });
        }
    };

    for attr in attrs {
        if attr.path.is_ident("schema") {
            for config in parse2::<Paren<Delimited<Meta>>>(attr.tokens.clone())
                .expect("invalid schema config found while extracting example")
                .inner
                .inner
            {
                match config {
                    Meta::NameValue(n) => process_nv(n),
                    Meta::List(l) => {
                        for el in l.nested {
                            match el {
                                NestedMeta::Meta(Meta::NameValue(n)) => process_nv(n),
                                _ => invalid_schema_usage!(attr),
                            }
                        }
                    }
                    _ => invalid_schema_usage!(attr),
                }
            }
        }
    }

    let v = v?;
    match syn::parse2::<Lit>(v.clone()) {
        Ok(v) => {
            let v = match v {
                Lit::Str(v) => q!(Vars { v }, { String(v.into()) }),
                Lit::ByteStr(_) => panic!("byte string is not a valid example"),
                Lit::Byte(_) => panic!("byte is not a valid example"),
                Lit::Char(v) => q!(Vars { v }, { String(v.into()) }),
                Lit::Int(v) => q!(Vars { v }, { Number(v.into()) }),
                Lit::Float(v) => q!(Vars { v }, { Number(v.into()) }),
                Lit::Bool(v) => q!(Vars { v }, { Bool(v) }),
                Lit::Verbatim(_) => unimplemented!("Verbatim?"),
            };

            Some(q!(Vars { v }, (rweb::rt::serde_json::Value::v)).into())
        }
        Err(..) => Some(v),
    }
}

fn extract_doc(attrs: &Vec<Attribute>) -> String {
    let mut doc = None;
    let mut comments = String::new();

    let mut process_doc_nv = |nv: syn::MetaNameValue| {
        if nv.path.is_ident("description") {
            if let Lit::Str(s) = nv.lit {
                doc = Some(s.value())
            } else {
                panic!(
                    "#[schema(description = \"foo\")]: value of example should be a string \
                     literal, but got {}",
                    nv.dump()
                )
            }
        }
    };

    for attr in attrs {
        if attr.path.is_ident("schema") {
            for config in parse2::<Paren<Delimited<Meta>>>(attr.tokens.clone())
                .expect("invalid schema config found while extracting example")
                .inner
                .inner
            {
                match config {
                    Meta::List(l) => {
                        for tag in l.nested {
                            match tag {
                                NestedMeta::Meta(Meta::NameValue(nv)) => process_doc_nv(nv),
                                _ => invalid_schema_usage!(attr),
                            }
                        }
                    }
                    Meta::NameValue(nv) => process_doc_nv(nv),
                    _ => invalid_schema_usage!(attr),
                }
            }
        } else if attr.path.is_ident("doc") {
            if let Ok(v) = parse2::<EqStr>(attr.tokens.clone()) {
                if !comments.is_empty() {
                    comments.push(' ');
                }
                comments.push_str(&v.value.value());
            };
        }
    }

    match doc {
        Some(v) => v,
        None => comments,
    }
}

fn handle_field(type_attrs: &[Attribute], f: &Field) -> Stmt {
    let name_str = field_name(type_attrs, &*f);

    let desc = extract_doc(&f.attrs);
    let example_v = extract_example(&f.attrs);
    let (skip_ser, skip_de) = get_skip_mode(&f.attrs);

    // We don't require it to be `Entity`
    if skip_ser && skip_de {
        return q!({ {} }).parse();
    }

    q!(
        Vars {
            name_str,
            desc,
            Type: &f.ty,
            example_v: super::quote_option(example_v),
            skip_ser,
            skip_de,
        },
        {
            fields.insert(rweb::rt::Cow::Borrowed(name_str), {
                {
                    #[allow(unused_mut)]
                    let mut s = <Type as rweb::openapi::Entity>::describe(comp_d);
                    if comp_d.get_unpack(&s).nullable != Some(true) {
                        required_fields.push(rweb::rt::Cow::Borrowed(name_str));
                    }
                    if let rweb::openapi::ComponentOrInlineSchema::Inline(s) = &mut s {
                        let description = desc;
                        if !description.is_empty() {
                            s.description = rweb::rt::Cow::Borrowed(description);
                        }
                        let example = example_v;
                        if let Some(example) = example {
                            s.example = Some(example);
                        }
                        if skip_ser {
                            s.write_only = Some(true);
                        }
                        if skip_de {
                            s.read_only = Some(true);
                        }
                    }
                    s
                }
            });
        }
    )
    .parse()
}

fn handle_fields(type_attrs: &[Attribute], fields: &Fields) -> Block {
    // Properties
    let mut block: Block = q!({ {} }).parse();
    block.stmts.push(
        q!({
            #[allow(unused_mut)]
            let mut fields: rweb::rt::IndexMap<rweb::rt::Cow<'static, str>, _> =
                rweb::rt::IndexMap::default();
        })
        .parse(),
    );
    block.stmts.push(
        q!({
            #[allow(unused_mut)]
            let mut required_fields: std::vec::Vec<rweb::rt::Cow<'static, str>> =
                std::vec::Vec::default();
        })
        .parse(),
    );

    for f in fields {
        block.stmts.push(handle_field(type_attrs, f));
    }

    block
        .stmts
        .push(Stmt::Expr(q!({ (fields, required_fields) }).parse()));

    block
}

fn extract_component(attrs: &[Attribute]) -> Option<String> {
    let mut component = None;
    let mut process_nv = |nv: syn::MetaNameValue| {
        if nv.path.is_ident("component") {
            if let Lit::Str(s) = nv.lit {
                assert!(
                    component.is_none(),
                    "duplicate #[schema(component = \"foo\")] detected"
                );
                component = Some(s.value())
            } else {
                panic!(
                    "#[schema]: value of component should be a string literal, but got {}",
                    nv.dump()
                )
            }
        } else {
            panic!("#[schema]: unknown option {}", nv.path.dump())
        }
    };
    for attr in attrs {
        if attr.path.is_ident("schema") {
            for config in parse2::<Paren<Delimited<Meta>>>(attr.tokens.clone())
                .expect("schema config of type is invalid")
                .inner
                .inner
            {
                match config {
                    Meta::List(l) => {
                        for el in l.nested {
                            match el {
                                syn::NestedMeta::Meta(Meta::NameValue(n)) => process_nv(n),
                                syn::NestedMeta::Meta(unk) => panic!(
                                    "#[schema]: parameters are name-value pair(s), but got {}",
                                    unk.dump()
                                ),
                                syn::NestedMeta::Lit(unk) => panic!(
                                    "#[schema]: parameters are name-value pair(s), but got {}",
                                    unk.dump()
                                ),
                            }
                        }
                    }
                    Meta::NameValue(nv) => process_nv(nv),
                    _ => panic!(
                        "#[schema]: parameters are name-value pair(s), but got {}",
                        config.dump()
                    ),
                }
            }
        }
    }
    component
}

pub fn derive_schema(input: DeriveInput) -> TokenStream {
    let DeriveInput {
        mut attrs,
        mut data,
        ident,
        generics,
        ..
    } = input;

    let desc = extract_doc(&mut attrs);

    let component = extract_component(&attrs);
    let example = extract_example(&mut attrs);

    let mut block: Block = q!({ {} }).parse();
    let mut fields: Punctuated<FieldValue, Token![,]> = Default::default();
    if let Some(tts) = example {
        fields.push(q!(Vars { tts }, ({ example: Some(tts) })).parse());
    }

    match data {
        Data::Struct(ref mut data) => {
            match data.fields {
                Fields::Named(_) => {
                    let fields_block = handle_fields(&attrs, &mut data.fields);
                    block.stmts.push(
                        q!(Vars { fields_block }, {
                            let (fields, required_fields) = fields_block;
                        })
                        .parse(),
                    );
                    fields.push(q!({ properties: fields }).parse());
                    fields.push(q!({ required: required_fields }).parse());
                }
                Fields::Unnamed(ref n) if n.unnamed.len() == 1 => {}
                _ => {}
            }

            fields.push(q!({ schema_type: Some(rweb::openapi::Type::Object) }).parse());
        }
        Data::Enum(ref mut data) => {
            let ett = get_enum_tag_type(&attrs);
            if data
                .variants
                .iter()
                .all(|variant| variant.fields.is_empty())
            {
                // c-like enums

                let exprs: Punctuated<Expr, Token![,]> = data
                    .variants
                    .iter()
                    .map(|variant| {
                        let name = if let Some(v) = get_rename(&variant.attrs) {
                            v
                        } else {
                            let rule = get_rename_all(&attrs);
                            rule.apply_to_variant(&variant.ident.to_string())
                        };
                        Pair::Punctuated(
                            q!(Vars { name }, { rweb::rt::Cow::Borrowed(name) }).parse(),
                            Default::default(),
                        )
                    })
                    .collect();

                match ett {
                    EnumTagType::External => {
                        fields.push(q!(Vars { exprs }, { enum_values: vec![exprs] }).parse());
                        fields.push(q!({ schema_type: Some(rweb::openapi::Type::String) }).parse());
                    }
                    EnumTagType::Internal { tag } | EnumTagType::Adjacent { tag, .. } => {
                        fields.push(q!({ schema_type: Some(rweb::openapi::Type::Object) }).parse());
                        fields.push(q!(Vars { exprs, tag: tag.as_str() }, {
                            properties: rweb::rt::indexmap![rweb::rt::Cow::Borrowed(tag) => rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                schema_type: Some(rweb::openapi::Type::String),
                                enum_values: vec![exprs],
                                ..rweb::rt::Default::default()
                            })]
                        }).parse());
                        fields.push(q!(Vars { tag }, {
                            required: vec![rweb::rt::Cow::Borrowed(tag)]
                        }).parse());
                    }
                    EnumTagType::None => panic!("Schema generation for C-Like enums with untagged representation is not supported")
                }
            } else {
                let variants: Vec<(String, Option<Expr>)> = data.variants.iter().filter_map(|v| {
                    let name = get_rename(&v.attrs).unwrap_or_else(|| get_rename_all(&attrs).apply_to_variant(&v.ident.to_string()));
                    let desc = extract_doc(&v.attrs);
                    match v.fields {
                        Fields::Named(..) => Some((name, Some({
                            let fields_block = handle_fields(&attrs, &v.fields);
                            q!(
                                Vars { fields_block, desc },
                                ({
                                    let (fields, fields_required) = fields_block;
                                    #[allow(unused_mut)]
                                    let mut s = rweb::openapi::Schema {
                                        schema_type: Some(rweb::openapi::Type::Object),
                                        properties: fields,
                                        required: fields_required,
                                        ..rweb::rt::Default::default()
                                    };
                                    let description = desc;
                                    if !description.is_empty() {
                                        s.description =
                                            rweb::rt::Cow::Borrowed(description);
                                    }

                                    rweb::openapi::ComponentOrInlineSchema::Inline(s)
                                })
                            )
                            .parse()
                        }))),
                        Fields::Unnamed(ref f) if f.unnamed.is_empty() => Some((name, None)),
                        Fields::Unnamed(ref f) if f.unnamed.len() == 1 => Some((name, Some(
                            q!(
                                Vars {
                                    Type: &f.unnamed.first().unwrap().ty,
                                    desc
                                },
                                ({
                                    #[allow(unused_mut)]
                                    let mut s =
                                        <Type as rweb::openapi::Entity>::describe(comp_d);
                                    if let rweb::openapi::ComponentOrInlineSchema::Inline(
                                        s,
                                    ) = &mut s
                                    {
                                        let description = desc;
                                        if !description.is_empty() {
                                            s.description =
                                                rweb::rt::Cow::Borrowed(description);
                                        }
                                    }

                                    s
                                })
                            )
                            .parse()
                        ))),
                        Fields::Unnamed(..) => panic!("Schema generation for tuple enum variants is currently not supported"),
                        Fields::Unit => Some((name, None)),
                    }
                }).collect();

                match ett {
                    EnumTagType::External => {
                        let variants: Punctuated<pmutil::Quote, Token![,]> = variants.into_iter().map(|(name, schema)| if let Some(schema) = schema {
                            q!(Vars { name, schema }, { rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                schema_type: Some(rweb::openapi::Type::Object),
                                properties: rweb::rt::indexmap![rweb::rt::Cow::Borrowed(name) => schema],
                                required: vec![rweb::rt::Cow::Borrowed(name)],
                                ..Default::default()
                            })})
                        } else {
                            q!(Vars { name }, { rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                schema_type: Some(rweb::openapi::Type::String),
                                enum_values: vec![rweb::rt::Cow::Borrowed(name)],
                                ..Default::default()
                            })})
                        }).collect();
                        fields.push(q!(Vars { variants }, { one_of: vec![variants] }).parse());
                    }
                    EnumTagType::Internal { tag } => {
                        let variants: Punctuated<pmutil::Quote, Token![,]> = variants.into_iter().map(|(name, schema)| if let Some(schema) = schema {
                            q!(Vars { tag: &tag, name, s: schema }, { rweb::openapi::ComponentOrInlineSchema::Inline(match s {
                                rweb::openapi::ComponentOrInlineSchema::Inline(mut schema) if schema.schema_type == Some(rweb::openapi::Type::Object) => {
                                    if schema.properties.insert(rweb::rt::Cow::Borrowed(tag), rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                        schema_type: Some(rweb::openapi::Type::String),
                                        enum_values: vec![rweb::rt::Cow::Borrowed(name)],
                                        ..Default::default()
                                    })).is_some() {
                                        panic!("Enum internal repr tag property interferes with property of enum variant");
                                    }
                                    schema.required.push(rweb::rt::Cow::Borrowed(tag));
                                    schema
                                }
                                schema => rweb::openapi::Schema {
                                    all_of: vec![rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                        schema_type: Some(rweb::openapi::Type::Object),
                                        properties: rweb::rt::indexmap![rweb::rt::Cow::Borrowed(tag) => rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                            schema_type: Some(rweb::openapi::Type::String),
                                            enum_values: vec![rweb::rt::Cow::Borrowed(name)],
                                            ..Default::default()
                                        })],
                                        required: vec![rweb::rt::Cow::Borrowed(tag)],
                                        ..Default::default()
                                    }), schema],
                                    ..Default::default()
                                }
                            })})
                        } else {
                            q!(Vars { tag: &tag, name }, { rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                schema_type: Some(rweb::openapi::Type::Object),
                                properties: rweb::rt::indexmap![rweb::rt::Cow::Borrowed(tag) => rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                    schema_type: Some(rweb::openapi::Type::String),
                                    enum_values: vec![rweb::rt::Cow::Borrowed(name)],
                                    ..Default::default()
                                })],
                                required: vec![rweb::rt::Cow::Borrowed(tag)],
                                ..Default::default()
                            })})
                        }).collect();
                        fields.push(q!(Vars { variants }, { one_of: vec![variants] }).parse());
                    }
                    EnumTagType::Adjacent { tag, content } => {
                        let variants: Punctuated<pmutil::Quote, Token![,]> = variants.into_iter().map(|(name, schema)| if let Some(schema) = schema {
                            q!(Vars { tag: &tag, content: &content, name, schema }, { rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                schema_type: Some(rweb::openapi::Type::Object),
                                properties: rweb::rt::indexmap![rweb::rt::Cow::Borrowed(tag) => rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                    schema_type: Some(rweb::openapi::Type::String),
                                    enum_values: vec![rweb::rt::Cow::Borrowed(name)],
                                    ..Default::default()
                                }), rweb::rt::Cow::Borrowed(content) => schema],
                                required: vec![rweb::rt::Cow::Borrowed(tag), rweb::rt::Cow::Borrowed(content)],
                                ..Default::default()
                            })})
                        } else {
                            q!(Vars { tag: &tag, name }, { rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                schema_type: Some(rweb::openapi::Type::Object),
                                properties: rweb::rt::indexmap![rweb::rt::Cow::Borrowed(tag) => rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                                    schema_type: Some(rweb::openapi::Type::String),
                                    enum_values: vec![rweb::rt::Cow::Borrowed(name)],
                                    ..Default::default()
                                })],
                                required: vec![rweb::rt::Cow::Borrowed(tag)],
                                ..Default::default()
                            })})
                        }).collect();
                        fields.push(q!(Vars { variants }, { one_of: vec![variants] }).parse());
                    }
                    EnumTagType::None => {
                        let variants: Punctuated<Expr, Token![,]> = variants.into_iter().map(|(_, schema)| schema.expect("Schema generation for unit variant in untagged enum is not supported")).collect();
                        fields.push(q!(Vars { variants }, { one_of: vec![variants] }).parse());
                    }
                }
            }
        }
        Data::Union(_) => unimplemented!("#[derive(Schema)] for union"),
    }

    block.stmts.push(Stmt::Expr(
        if component.is_some() {
            q!(Vars { desc, fields }, {
                comp_d.describe_component(&Self::type_name(), |comp_d| rweb::openapi::Schema {
                    fields,
                    description: rweb::rt::Cow::Borrowed(desc),
                    ..rweb::rt::Default::default()
                })
            })
        } else {
            q!(Vars { desc, fields }, {
                rweb::openapi::ComponentOrInlineSchema::Inline(rweb::openapi::Schema {
                    fields,
                    description: rweb::rt::Cow::Borrowed(desc),
                    ..rweb::rt::Default::default()
                })
            })
        }
        .parse(),
    ));

    let typename = component.clone().unwrap_or_else(|| ident.to_string());
    let typename: Expr = if generics.params.is_empty() {
        q!(Vars { typename }, { rweb::rt::Cow::Borrowed(typename) }).parse()
    } else {
        let generics_typenames: Punctuated<pmutil::Quote, Token![,]> = generics
            .params
            .iter()
            .flat_map(|g| match g {
                syn::GenericParam::Type(t) => Some({
                    let tpn = &t.ident;
                    q!(Vars { tpn }, {
                        {
                            <tpn as rweb::openapi::Entity>::type_name().to_string()
                        }
                    })
                }),
                syn::GenericParam::Const(con) => Some({
                    let tpn = &con.ident;
                    q!(Vars { tpn }, {
                        {
                            tpn.to_string()
                        }
                    })
                }),
                _ => None,
            })
            .map(|q| Pair::Punctuated(q, Default::default()))
            .collect();
        q!(
            Vars {
                typename,
                generics_typenames
            },
            {
                rweb::rt::Cow::Owned(format!(
                    "{}-{}-",
                    typename,
                    vec![generics_typenames].join("_")
                ))
            }
        )
        .parse()
    };

    let mut item = q!(
        Vars {
            Type: &ident,
            typename,
            block,
        },
        {
            impl rweb::openapi::Entity for Type {
                fn type_name() -> rweb::rt::Cow<'static, str> {
                    typename
                }

                fn describe(
                    comp_d: &mut rweb::openapi::ComponentDescriptor,
                ) -> rweb::openapi::ComponentOrInlineSchema {
                    block
                }
            }
        }
    )
    .parse::<ItemImpl>()
    .with_generics(generics);

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
