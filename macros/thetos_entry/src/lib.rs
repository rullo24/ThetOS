use proc_macro::TokenStream; // allows us to parse and manipulate tokens

mod entry; // contains the implementation of the #[entry] macro

#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    entry::expand_thetos_entry(attr, item)
}