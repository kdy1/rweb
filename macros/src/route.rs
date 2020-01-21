use crate::ParenTwoValue;
use pmutil::{q, ToTokensExt};
use proc_macro2::Ident;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse2, Attribute, Error, Expr, Meta, MetaNameValue, NestedMeta, Token,
};

/// A node wrapped with paren.
struct Paren<T> {
    inner: T,
}

impl<T> Parse for Paren<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let content;
        parenthesized!(content in input);
        Ok(Paren {
            inner: content.parse()?,
        })
    }
}

/// Handle attributes on fn item like `#[header(ContentType =
/// "application/json")]`
pub fn compile_item_attrs(mut base: Expr, attrs: &mut Vec<Attribute>) -> Expr {
    attrs.retain(|attr| {
        if attr.path.is_ident("header") {
            let t: ParenTwoValue = parse2(attr.tokens.clone()).expect(
                "failed to parser header. Please provide it like #[header(\"ContentType\", \
                 \"application/json\")]",
            );

            base = q!(
                Vars {
                    base: &base,
                    k: t.key,
                    v: t.value
                },
                { base.and(rweb::header::exact_ignore_case(k, v)) }
            )
            .parse();
            return false;
        }

        if attr.path.is_ident("body_size") {
            let meta = parse2::<Paren<MetaNameValue>>(attr.tokens.clone())
                .expect("Correct usage: #[body_size(max = \"8192\")]")
                .inner;

            if meta.path.is_ident("max") {
                let v = meta.lit.dump().to_string();
                let mut value = &*v;
                if value.starts_with('"') {
                    value = &value[1..value.len() - 1];
                }
                let tts: proc_macro2::TokenStream = value.parse().unwrap_or_else(|err| {
                    panic!(
                        "#[body_size]: failed to parse value of max as number: {:?}",
                        err
                    )
                });

                base = q!(
                    Vars {
                        base: &base,
                        v: tts
                    },
                    { base.and(rweb::filters::body::content_length_limit(v)) }
                )
                .parse();
                return false;
            } else {
                panic!("Unknown configuration {} for #[body_size]", meta.dump())
            }
        }

        true
    });

    base
}
