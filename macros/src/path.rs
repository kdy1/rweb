use pmutil::q;
use proc_macro2::TokenStream;
use syn::{parse_quote::parse, punctuated::Punctuated, Expr, FnArg, LitStr, Pat, Signature, Token};

///
/// - `sig`: sohuld be [Some] only if path parameters are allowed
pub fn compile(
    base: Option<Expr>,
    path: TokenStream,
    sig: Option<&Signature>,
    end: bool,
) -> (Expr, Vec<(String, usize)>) {
    let path: LitStr = parse(path);
    let path = path.value();
    assert!(path.starts_with('/'), "Path should start with /");

    let segments = path.split('/');
    let len = segments.clone().filter(|&s| s != "").count();

    if len == 0 {
        return (
            q!({ rweb::filters::path::end() }).parse(),
            Default::default(),
        );
    }

    let mut exprs: Punctuated<Expr, Token![.]> = Default::default();
    exprs.extend(base);
    let mut vars = vec![];

    for segment in segments {
        if segment == "" {
            continue;
        }

        let expr = if segment.starts_with('{') {
            let v = &segment[1..segment.len() - 1];

            if let Some(sig) = sig {
                let ty = sig
                    .inputs
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, arg)| match arg {
                        FnArg::Typed(ty) => match *ty.pat {
                            Pat::Ident(ref i) if i.ident == v => {
                                vars.push((v.to_string(), idx));
                                Some(&ty.ty)
                            }
                            _ => None,
                        },

                        _ => None,
                    })
                    .next()
                    .unwrap_or_else(|| panic!("failed to find parameter named `{}`", v));

                q!(Vars { ty }, { rweb::filters::path::param::<ty>() })
            } else {
                panic!("path parameters are not allowed here (currently)")
            }
        } else {
            q!(Vars { segment }, { rweb::filters::path::path(segment) })
        };

        if exprs.is_empty() {
            exprs.push(q!(Vars { expr }, { expr }).parse());
        } else {
            exprs.push(q!(Vars { expr }, { and(expr) }).parse());
        }
    }

    if end {
        exprs.push(q!({ and(rweb::filters::path::end()) }).parse());
    }

    (q!(Vars { exprs }, { exprs }).parse(), vars)
}
