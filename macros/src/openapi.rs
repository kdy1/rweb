use crate::parse::{Delimited, Paren};
use pmutil::ToTokensExt;
use rweb_openapi::v3_0::Operation;
use syn::{parse2, Attribute, Meta};

pub fn parse(attrs: &mut Vec<Attribute>) -> Operation {
    let mut op = Operation::default();
    op.description = Some(String::new());

    attrs.retain(|attr| {
        if attr.path.is_ident("openapi") {
            // tags("foo", "bar", "baz)

            let configs = parse2::<Paren<Delimited<Meta>>>(attr.tokens.clone())
                .expect("openapi config is invalid")
                .inner
                .inner;

            for config in configs {
                if config.path().is_ident("tags") {
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
