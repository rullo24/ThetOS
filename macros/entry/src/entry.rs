use proc_macro::{
    Ident, // used to compare Ident to string
    TokenStream, // used to parse and manipulate tokens
    TokenTree // used to iterate over tokens
};

/// DESCRIPTION
/// expands #[entry(bsp = <bsp_crate>)] to use bsp, link main and the annotated function
pub(crate) fn expand_thetos_entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    
    // `attr` is only the inside of the attribute, e.g. `bsp = nucleo_l152re` (no `#[` `]`).
    let bsp_crate: String = match parse_bsp_attr(attr) {
        Ok(s) => s,
        Err(msg) => return compile_error_tokens(msg),
    };

    // force the board crate to link against the target
    let prelude: String = format!("use {bsp_crate} as _;");
    let mut out: TokenStream = match parse_tokens(&prelude) {
        Some(t) => t,
        None => return compile_error_tokens("entry: internal parse error (prelude)"),
    };

    // annotated item must be a function -> capture the name
    let fn_name = match fn_name_after_keyword_fn(&item) {
        Some(n) => n,
        None => return compile_error_tokens("entry: expected a `fn name(...) -> ...` item"),
    };

    // `main` is reserved for the linker -> ensure users can't force this
    if fn_name == "main" {
        return compile_error_tokens("entry: `main` is reserved for the linker; use a diff entry name.");
    }

    // shim the annotated function to be the entry point
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
/// parses the bsp, returning the crate name as a string (or an error message)
fn parse_bsp_attr(attr: TokenStream) -> Result<String, &'static str> {
    let mut it = attr.into_iter();
    
    // first token must be the `bsp` keyword
    match it.next() {
        Some(TokenTree::Ident(i)) if ident_matches(&i, "bsp") => {}
        _ => return Err("entry: expected `bsp = <crate>`"),
    }

    // next token must be the `=` operator
    match it.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => {}
        _ => return Err("entry: expected `=` after `bsp`"),
    }

    // next token must be the crate name
    let name: String = match it.next() {
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

    // should be no trailing tokens -> error if there are
    if it.next().is_some() {
        return Err("entry: trailing tokens in attribute");
    }

    // crate name must be a valid Rust identifier -> error otherwise
    if !is_safe_crate_token(&name) {
        return Err("entry: invalid `bsp` crate name");
    }

    return Ok(name);
}

/// DESCRIPTION
/// checks if a string is a valid Rust crate name
fn is_safe_crate_token(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

/// DESCRIPTION
/// walks the funct item and returns the keyword after the first `fn` keyword (e.g. `app_main` for `pub fn app_main`)
fn fn_name_after_keyword_fn(item: &TokenStream) -> Option<String> {
    let mut after_fn = false;
    
    // iterate over the tokens
    for tt in item.clone() {
        if let TokenTree::Ident(i) = tt {
            
            // if we're after the `fn` keyword, return the name
            if after_fn {
                return Some(i.to_string());
            }
            
            if ident_matches(&i, "fn") {
                after_fn = true;
            }
        }
    }
    return None;
}

/// DESCRIPTION
/// compares an `Ident` to a string -> use strings because stable Rust has no `Ident == &str`.
fn ident_matches(i: &Ident, s: &str) -> bool {
    return i.to_string() == s;
}

/// DESCRIPTION
/// parse Rust source snippet to tokens
fn parse_tokens(src: &str) -> Option<TokenStream> {
    return src.parse().ok();
}

/// DESCRIPTION
/// captures tokens that make rustc report `msg` + abort crate compilation
fn compile_error_tokens(msg: &str) -> TokenStream {
    format!("compile_error!({:?});", msg)
        .parse()
        .unwrap_or_else(|_| TokenStream::new())
}
