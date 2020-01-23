use super::ParenTwoValue;
use crate::parse::{Delimited, Paren};
use pmutil::{q, ToTokensExt};
use syn::{parse2, Attribute, Expr, Meta, MetaNameValue};

/// Handle attributes on fn item like `#[header(ContentType =
/// "application/json")]`
pub fn compile_fn_attrs(mut base: Expr, attrs: &mut Vec<Attribute>, emitted_map: bool) -> Expr {
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
            }

            panic!("Unknown configuration {} for #[body_size]", meta.dump())
        }

        if attr.path.is_ident("cors") && emitted_map {
            let correct_usage =
                "Correct usage:\n#[cors(origins(\"example.com\", \"your.site.com\"), methods(get, \
                 post), headers(\"accept\"), max_age = 600)]\nNote: origins(\"*\") can be used to \
                 indicate cors is allowed for all origin. \nNote: you can omit methods to allow \
                 all methods.\nNote: you can omit headers to use the default behavior.\nNote: you \
                 can omit max_age to use the default value";

            let configs = parse2::<Paren<Delimited<Meta>>>(attr.tokens.clone())
                .expect(correct_usage)
                .inner
                .inner;

            let mut cors_expr: Expr = q!({ rweb::filters::cors::cors() }).parse();

            for config in configs {
                match config {
                    Meta::Path(p) => unimplemented!("Path meta: {}\n{}", p.dump(), correct_usage),
                    Meta::List(l) => {
                        if l.path.is_ident("origins") {
                            for origin in l.nested {
                                // TODO: More verification
                                let is_wildcard = origin.dump().to_string() == "\"*\"";

                                if is_wildcard {
                                    cors_expr =
                                        q!(Vars { cors_expr }, { cors_expr.allow_any_origin() })
                                            .parse();
                                } else {
                                    cors_expr = q!(Vars { cors_expr, origin }, {
                                        cors_expr.allow_origin(origin)
                                    })
                                    .parse();
                                }
                            }
                        } else if l.path.is_ident("methods") {
                            // TODO: More verification (namely string literal)

                            for method in l.nested {
                                cors_expr = q!(Vars { cors_expr, method }, {
                                    cors_expr.allow_method(stringify!(method))
                                })
                                .parse();
                            }
                        } else if l.path.is_ident("headers") {
                            for header in l.nested {
                                cors_expr = q!(Vars { cors_expr, header }, {
                                    cors_expr.allow_header(header)
                                })
                                .parse();
                            }
                        } else {
                            panic!("Unknown config: `{}`\n{}", l.dump(), correct_usage)
                        }
                    }
                    Meta::NameValue(n) => {
                        if n.path.is_ident("max_age") {
                            cors_expr = q!(
                                Vars {
                                    cors_expr,
                                    v: n.lit
                                },
                                { cors_expr.max_age(v) }
                            )
                            .parse();
                        } else {
                            panic!("Unknown config: `{}`\n{}", n.dump(), correct_usage)
                        }
                    }
                }
            }

            base = q!(
                Vars {
                    base: &base,
                    cors_expr
                },
                { base.with(cors_expr.build()) }
            )
            .parse();

            return false;
        }

        true
    });

    base
}
