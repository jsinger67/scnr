use proc_macro::TokenStream;

#[proc_macro]
pub fn scanner(_input: TokenStream) -> TokenStream {
    TokenStream::default()
}
