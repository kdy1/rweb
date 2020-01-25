use pmutil::q;
use proc_macro2::TokenStream;
use syn::{
    parse_quote::parse, punctuated::Punctuated, Expr, FnArg, LitStr, Pat, Signature, Token, Type,
};

pub fn find_ty<'a>(sig: &'a Signature, name: &str) -> Option<&'a Type> {
    sig.inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(ty) => match *ty.pat {
                Pat::Ident(ref i) if i.ident == name => Some(&*ty.ty),
                _ => None,
            },

            _ => None,
        })
        .next()
}

///
/// - `sig`: Should be [Some] only if path parameters are allowed
pub fn compile(
    base: Option<Expr>,
    path: TokenStream,
    sig: Option<&Signature>,
    end: bool,
) -> (Expr, Vec<(String, usize)>) {
    let path: LitStr = parse(path);
    let path = path.value();
    assert!(path.starts_with('/'), "Path should start with /");
    assert!(
        path.find("//").is_none(),
        "A route containing `//` doesn't make sense"
    );

    let mut exprs: Punctuated<Expr, Token![.]> = Default::default();
    // Set base values
    exprs.extend(base);
    let mut vars = vec![];

    // Filter empty segments before iterating over them.
    // Mainly it will come from the required path in the beginning / but could also come from the end /
    // Example: #[get("/{word}")] or #[get("/{word}/")] with the `/` before and after `{word}`
    let segments: Vec<&str> = path.split('/').into_iter().filter(|&x| x != "").collect();
    for segment in segments {
        let expr = if segment.starts_with('{') {
            // Example if {word} we only want to extract `word` here
            let name = &segment[1..segment.len() - 1];
            if let Some(sig) = sig {
                let ty = sig
                    .inputs
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, arg)| match arg {
                        FnArg::Typed(ty) => match *ty.pat {
                            // Here if we find a Pat::Ident we get i: &PatIdent and i.ident is the parameter in the route fn.
                            // I.e dyn_reply(word: String), this would be named: `word` and we compare it to the segment name mentioned above.
                            // If they match:
                            //      We uses it and adds to our variables.
                            // else
                            //      We will panic later.
                            Pat::Ident(ref i) if i.ident == name => {
                                vars.push((name.to_string(), idx));
                                Some(&ty.ty)
                            }
                            _ => None,
                        },

                        _ => None,
                    })
                    .next()
                    .unwrap_or_else(|| panic!("failed to find parameter named `{}`", name));

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
