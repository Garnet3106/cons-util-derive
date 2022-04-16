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
    let mut variant_ptts = quote!{};

    match option.data {
        darling::ast::Data::Enum(variant_options) => {
            for each_variant_option in variant_options {
                let variant_name = each_variant_option.ident;
                let mut lang_ptts = quote!{};

                macro_rules! append_impl {
                    ($lang_name:expr, $field_name:ident) => {
                        match each_variant_option.$field_name {
                            Some(translation_result) => {
                                let new_lang_ptt = quote!{
                                    $lang_name => #translation_result,
                                };

                                lang_ptts.append_all(vec![new_lang_ptt]);
                            },
                            None => (),
                        }
                    };
                }

                append_impl!("en", en);
                append_impl!("ja", ja);

                let new_variant_ptt = quote!{
                    #enum_name::#variant_name => {
                        match lang {
                            #lang_ptts
                            _ => "<UNKNOWN_LANGUAGE>",
                        }
                    },
                };

                variant_ptts.append_all(vec![new_variant_ptt]);
            }
        },
        _ => panic!("Expected an enum."),
    };

    let fn_ident = Ident::new("translate", Span::call_site());

    let gen = quote!{
        impl #enum_name {
            pub fn #fn_ident(&self, lang: &str) -> &str {
                return match self {
                    #variant_ptts
                    _ => "<UNKNOWN_LOG>",
                };
            }
        }
    };

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
    ident: Ident,
    #[darling(default)]
    en: Option<String>,
    #[darling(default)]
    ja: Option<String>,
}
