use crate::parse::{Delimited, Paren};
use pmutil::{q, ToTokensExt};
use rweb_openapi::v3_0::Operation;
use syn::{parse2, Attribute, Expr, Lit, Meta};

pub fn quote_op(op: Operation) -> Expr {
    q!(
        Vars {
            id_v: op.operation_id,
            summary_v: op.summary,
        },
        {
            rweb::openapi::Operation {
                tags: Default::default(),
                summary: summary_v.to_string(),
                description: Default::default(),
                external_docs: Default::default(),
                operation_id: id_v.to_string(),
                parameters: Default::default(),
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

pub fn parse(attrs: &mut Vec<Attribute>) -> Operation {
    let mut op = Operation::default();

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
                        Meta::Path(_) => unreachable!(),
                        Meta::List(_) => panic!("Correct usage: #[openapi(id = \"foo\")]"),
                        Meta::NameValue(v) => match v.lit {
                            Lit::Str(s) => op.operation_id = s.value(),
                            _ => panic!("#[openapi]: invalid operation id"),
                        },
                    }
                } else if config.path().is_ident("summary") {
                    match config {
                        Meta::Path(_) => unreachable!(),
                        Meta::List(_) => panic!("Correct usage: #[openapi(summary = \"foo\")]"),
                        Meta::NameValue(v) => match v.lit {
                            Lit::Str(s) => op.summary = s.value(),
                            _ => panic!("#[openapi]: invalid operation summary"),
                        },
                    }
                } else if config.path().is_ident("tags") {
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
