extern crate proc_macro;

use {
    darling::*,
    darling::ast::Data,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::*,
    syn::*,
};

#[proc_macro_derive(LogTranslation, attributes(translate))]
pub fn derive_log(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let option = LogTranslationOption::from_derive_input(&item).expect("Failed to parse derive input.");
    let enum_name = item.ident;
    let mut fns = quote!{};

    match option.data {
        darling::ast::Data::Enum(variant_options) => {
            for each_variant_option in variant_options {
                macro_rules! append_impl {
                    ($lang_name:expr, $field_name:ident) => {
                        match each_variant_option.$field_name {
                            Some(translation_result) => {
                                let fn_ident = Ident::new(&format!("translate_{}_to_{}", option.ident, $lang_name), Span::call_site());
                                let new_gen = quote!{
                                    pub fn #fn_ident() -> String {
                                        return #translation_result.to_string();
                                    }
                                };

                                fns.append_all(vec![new_gen]);
                            },
                            None => (),
                        }
                    };
                }

                append_impl!("en", en);
                append_impl!("ja", ja);
            }
        },
        _ => panic!("Expected an enum."),
    };

    let gen = quote!{impl #enum_name {#fns}};
    return gen.into();
}

#[derive(Clone, Debug, FromDeriveInput)]
#[darling(attributes(translate))]
struct LogTranslationOption {
    ident: Ident,
    data: Data<LogTranslationVariantOption, ()>,
}

#[derive(Clone, Debug, FromVariant)]
#[darling(attributes(translate))]
struct LogTranslationVariantOption {
    #[darling(default)]
    en: Option<String>,
    #[darling(default)]
    ja: Option<String>,
}
