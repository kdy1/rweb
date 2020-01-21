use crate::ParenTwoValue;
use pmutil::q;
use syn::{parse2, Attribute, Expr};

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

        true
    });

    base
}
