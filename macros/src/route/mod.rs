use pmutil::{q, Quote, ToTokensExt};
use proc_macro2::TokenStream;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote::parse,
    punctuated::Punctuated,
    visit::Visit,
    Block, Expr, ItemFn, LitStr, ReturnType, Signature, Token, Type, TypeImplTrait, Visibility,
};

pub mod fn_attr;
pub mod param;

/// An eq token followed by literal string
pub(crate) struct EqStr {
    _eq: Token![=],
    pub value: LitStr,
}

impl Parse for EqStr {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        Ok(EqStr {
            _eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

/// An eq token followed by literal string
pub(crate) struct ParenTwoValue {
    key: LitStr,
    _eq: Token![,],
    value: LitStr,
}

impl Parse for ParenTwoValue {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let content;
        parenthesized!(content in input);
        Ok(ParenTwoValue {
            key: content.parse()?,
            _eq: content.parse()?,
            value: content.parse()?,
        })
    }
}

pub fn compile_route(
    method: Option<Quote>,
    path: TokenStream,
    f: TokenStream,
) -> proc_macro::TokenStream {
    let mut f: ItemFn = parse(f);
    let sig = &f.sig;
    let mut data_inputs: Punctuated<_, Token![,]> = Default::default();

    // Apply method filter
    let expr: Expr = if let Some(ref method) = method {
        q!(
            Vars {
                http_method: method,
            },
            { rweb::filters::method::http_method() }
        )
        .parse()
    } else {
        q!({ rweb::filters::any() }).parse()
    };

    let (mut expr, vars) = crate::path::compile(Some(expr), path.clone(), Some(sig), true);
    let path: LitStr = parse(path);
    let path = path.value();

    let (handler_fn, from_req_types) = {
        let (e, inputs, from_req_types) =
            param::compile(expr, &f.sig, &mut data_inputs, vars, true);
        expr = e;
        (
            ItemFn {
                attrs: Default::default(),
                vis: Visibility::Inherited,

                sig: Signature {
                    //                asyncness: None,
                    inputs,
                    ..f.sig.clone()
                },
                block: f.block,
            },
            from_req_types,
        )
    };

    let should_use_impl_trait =
        sig.asyncness.is_some() || f.attrs.iter().any(|attr| attr.path.is_ident("cors"));

    let expr = fn_attr::compile_fn_attrs(expr, &mut f.attrs, false);

    let expr = if sig.asyncness.is_some() {
        q!(
            Vars {
                handler: &sig.ident,
                expr
            },
            { expr.and_then(handler) }
        )
    } else {
        q!(
            Vars {
                handler: &sig.ident,
                expr
            },
            { expr.map(handler) }
        )
    }
    .parse::<Expr>();

    let mut expr = fn_attr::compile_fn_attrs(expr, &mut f.attrs, true);

    let ret = if should_use_impl_trait {
        q!((impl rweb::Reply)).dump()
    } else {
        match sig.output {
            ReturnType::Default => panic!("http handler should return type"),
            ReturnType::Type(_, ref ty) => ty.dump(),
        }
    };

    if cfg!(feature = "openapi") {
        let op = crate::openapi::parse(&path, sig, &mut f.attrs);
        let op = crate::openapi::quote_op(op);

        let mut op_body: Block = q!(Vars { op }, {
            {
                #[allow(unused_mut)]
                let mut v = op;
            }
        })
        .parse();

        for from_req in from_req_types {
            op_body.stmts.push(
                q!(Vars { Type: &from_req }, {
                    rweb::openapi::Collector::add_request_type_to::<Type>(__collector, &mut v);
                })
                .parse(),
            );
        }

        match sig.output {
            ReturnType::Default => panic!("http handlers should have return type"),
            ReturnType::Type(_, ref ty) => {
                if !contains_impl_trait(&**ty) {
                    op_body.stmts.push(
                        q!(Vars { Type: ty }, {
                            rweb::openapi::Collector::add_response_to::<Type>(__collector, &mut v);
                        })
                        .parse(),
                    );
                }
            }
        }

        op_body.stmts.push(
            q!(
                Vars {
                    path: &path,
                    http_method: method,
                },
                {
                    __collector.add(path, rweb::openapi::http_methods::http_method(), v);
                }
            )
            .parse(),
        );

        expr = q!(Vars { expr, op_body }, {
            rweb::openapi::with(|__collector: Option<&mut rweb::openapi::Collector>| {
                if let Some(__collector) = __collector {
                    op_body
                }

                expr
            })
        })
        .parse();
    }

    let mut outer = if cfg!(feature = "boxed") {
        q!(
            Vars {
                expr,
                handler: &sig.ident,
                Ret: ret,
                handler_fn,
            },
            {
                fn handler(
                ) -> rweb::filters::BoxedFilter<(Ret,)> + rweb::rt::Clone {
                    use rweb::Filter;

                    handler_fn

                    expr.boxed()
                }
            }
        )
        .parse::<ItemFn>()
    } else {
        q!(
            Vars {
                expr,
                handler: &sig.ident,
                Ret: ret,
                handler_fn,
            },
            {
                fn handler(
                ) -> impl rweb::Filter<Extract = (Ret,), Error = rweb::warp::Rejection>
                       + rweb::rt::Clone {
                    use rweb::Filter;

                    handler_fn

                    expr
                }
            }
        )
        .parse::<ItemFn>()
    };

    outer.vis = f.vis;
    outer.sig = Signature {
        inputs: data_inputs,
        ..outer.sig
    };

    outer.dump().into()
}

fn contains_impl_trait(ty: &Type) -> bool {
    struct Visitor(bool);
    impl<'a> syn::visit::Visit<'a> for Visitor {
        fn visit_type_impl_trait(&mut self, _: &TypeImplTrait) {
            self.0 = true;
        }
    }

    let mut v = Visitor(false);

    v.visit_type(ty);

    v.0
}
