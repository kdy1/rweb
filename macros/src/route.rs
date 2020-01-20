use crate::ParenTwoValue;
use pmutil::{q, ToTokensExt};
use syn::{parse2, Attribute, Expr};

/// Handle attributes on fn item like `#[header(ContentType =
/// "application/json")]`
pub fn compile_item_attrs(mut base: Expr, attrs: Vec<Attribute>) -> Expr {
    for attr in attrs {
        if attr.path.is_ident("cfg") || attr.path.is_ident("doc") {
            continue;
        }

        if attr.path.is_ident("header") {
            let t: ParenTwoValue = parse2(attr.tokens).expect(
                "failed to parser header. Please provide it like #[header(\"ContentType\", \
                 \"application/json\")]",
            );

            base = q!(
                Vars {
                    base,
                    k: t.key,
                    v: t.value
                },
                { base.and(rweb::header::exact_ignore_case(k, v)) }
            )
            .parse();
        } else {
            panic!("Unknown attribute: {}", attr.dump())
        }
    }

    base
}
