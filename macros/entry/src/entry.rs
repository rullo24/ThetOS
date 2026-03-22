use proc_macro::{Ident, TokenStream, TokenTree};

/// DESCRIPTION
/// Expands `#[entry(bsp = …)]` into `use <bsp> as _`, linker `main`, and the annotated function.
pub(crate) fn expand_thetos_entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let bsp_crate = match parse_bsp_attr(attr) {
        Ok(s) => s,
        Err(msg) => return compile_error_tokens(msg),
    };

    let prelude = format!("use {bsp_crate} as _;");
    let mut out = match parse_tokens(&prelude) {
        Some(t) => t,
        None => return compile_error_tokens("entry: internal parse error (prelude)"),
    };

    let fn_name = match fn_name_after_keyword_fn(&item) {
        Some(n) => n,
        None => return compile_error_tokens("entry: expected a `fn name(...) -> ...` item"),
    };

    let shim = format!("#[no_mangle] pub extern \"C\" fn main() -> ! {{ {fn_name}() }}");
    let shim_ts = match parse_tokens(&shim) {
        Some(t) => t,
        None => return compile_error_tokens("entry: internal parse error (shim)"),
    };

    out.extend(shim_ts);
    out.extend(item);
    out
}

/// DESCRIPTION
/// Parses `bsp = <ident>` or `bsp = "…"` and returns the crate name string, or a static error message.
fn parse_bsp_attr(attr: TokenStream) -> Result<String, &'static str> {
    let mut it = attr.into_iter();
    match it.next() {
        Some(TokenTree::Ident(i)) if ident_matches(&i, "bsp") => {}
        _ => return Err("entry: expected `bsp = <crate>`"),
    }
    match it.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => {}
        _ => return Err("entry: expected `=` after `bsp`"),
    }
    let name = match it.next() {
        Some(TokenTree::Ident(i)) => i.to_string(),
        Some(TokenTree::Literal(l)) => {
            let raw = l.to_string();
            if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
                raw[1..raw.len() - 1].to_string()
            } else {
                return Err("entry: expected `\"crate_name\"` or bare ident after `bsp =`");
            }
        }
        _ => return Err("entry: expected crate name after `bsp =`"),
    };
    if it.next().is_some() {
        return Err("entry: trailing tokens in attribute");
    }
    if !is_safe_crate_token(&name) {
        return Err("entry: invalid `bsp` crate name");
    }
    Ok(name)
}

/// DESCRIPTION
/// True when `s` is non-empty and only ASCII alphanumerics or `_` (safe single-segment crate token).
fn is_safe_crate_token(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// DESCRIPTION
/// Returns the identifier immediately after the first `fn` keyword (e.g. `app_main` for `pub fn app_main`).
fn fn_name_after_keyword_fn(item: &TokenStream) -> Option<String> {
    let mut after_fn = false;
    for tt in item.clone() {
        if let TokenTree::Ident(i) = tt {
            if after_fn {
                return Some(i.to_string());
            }
            if ident_matches(&i, "fn") {
                after_fn = true;
            }
        }
    }
    None
}

/// DESCRIPTION
/// Compares a proc-macro `Ident` to `s` by stringising; stable Rust has no `Ident == &str`.
fn ident_matches(i: &Ident, s: &str) -> bool {
    i.to_string() == s
}

/// DESCRIPTION
/// Parses a Rust source snippet into a `TokenStream`, or `None` if the lexer rejects it.
fn parse_tokens(src: &str) -> Option<TokenStream> {
    src.parse().ok()
}

/// DESCRIPTION
/// Builds a `compile_error!(…);` token stream so expansion fails with a readable compiler error.
fn compile_error_tokens(msg: &str) -> TokenStream {
    format!("compile_error!({:?});", msg)
        .parse()
        .unwrap_or_else(|_| TokenStream::new())
}
