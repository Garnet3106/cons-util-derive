extern crate proc_macro;

use {
    proc_macro::TokenStream,
    quote::*,
    syn::*,
};

#[proc_macro_derive(Log, attributes(option))]
pub fn derive_log(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemEnum);
    let enum_name = item.ident;

    let gen = quote! {
        impl #enum_name {
            pub fn self_name(&self) -> &str {
                stringify!(#enum_name)
            }
        }
    };
    gen.into()
}
