extern crate proc_macro;

use {
    darling::*,
    darling::ast::Data,
    proc_macro::TokenStream,
    proc_macro2::Span,
    quote::*,
    regex::Regex,
    syn::*,
};

#[proc_macro_derive(LogTranslation, attributes(translate))]
pub fn derive_log(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let option = LogTranslationOption::from_derive_input(&item).expect("Failed to parse derive input.");
    let enum_name = item.ident;
    let mut variant_patts = quote!{};

    match option.data {
        darling::ast::Data::Enum(variant_options) => {
            for each_variant_option in &variant_options {
                let variant_name = &each_variant_option.ident;
                let (variant_field_idents, variant_field_tokens) = get_fields_from_variant_option(each_variant_option);
                let mut lang_patts = quote!{};

                macro_rules! append_impl {
                    ($lang_name:expr, $field_name:ident) => {
                        match &each_variant_option.$field_name {
                            Some(translation_result) => {
                                let fmt_translation_result = get_translation_result_formatter(translation_result, &variant_field_idents);

                                let new_lang_patt = quote!{
                                    $lang_name => #fmt_translation_result,
                                };

                                lang_patts.append_all(vec![new_lang_patt]);
                            },
                            None => (),
                        }
                    };
                }

                append_impl!("en", en);
                append_impl!("ja", ja);

                let new_variant_patt = quote!{
                    #enum_name::#variant_name #variant_field_tokens => {
                        match lang {
                            #lang_patts
                            _ => "<UNKNOWN_LANGUAGE>".to_string(),
                        }
                    },
                };

                variant_patts.append_all(vec![new_variant_patt]);
            }
        },
        _ => panic!("Expected an enum."),
    };

    let fn_ident = Ident::new("translate", Span::call_site());

    let gen = quote!{
        impl #enum_name {
            pub fn #fn_ident(&self, lang: &str) -> String {
                #[allow(unused_variables)]
                return match self {
                    #variant_patts
                    _ => "<UNKNOWN_LOG>".to_string(),
                };
            }
        }
    };

    return gen.into();
}

fn get_fields_from_variant_option(variant_option: &LogTranslationVariantOption) -> (Vec<String>, proc_macro2::TokenStream) {
    let mut is_tuple_field = false;
    let mut variant_field_idents = Vec::<String>::new();
    let mut variant_field_tokens = quote!{};

    for (field_i, each_field) in variant_option.fields.iter().enumerate() {
        is_tuple_field = each_field.ident.is_none();

        let variant_ident_str = match &each_field.ident {
            Some(ident) => ident.to_string(),
            None => format!("v{}", field_i),
        };

        variant_field_idents.push(variant_ident_str.to_string());
        let variant_ident = Ident::new(&variant_ident_str, Span::call_site());

        let new_variant_field = quote!{
            #variant_ident,
        };

        variant_field_tokens.append_all(vec![new_variant_field]);
    }

    let variant_field_tokens = if is_tuple_field {
        quote!{
            (#variant_field_tokens)
        }
    } else {
        quote!{
            { #variant_field_tokens }
        }
    };

    return (variant_field_idents, variant_field_tokens);
}

fn get_translation_result_formatter(translation_result: &str, variant_field_idents: &Vec<String>) -> proc_macro2::TokenStream {
    let fmt_regex_patt = Regex::new(r"\{(?:(?:[a-zA-Z_][a-zA-Z0-9_]*|\d+)?)\}").expect("Regex pattern is invalid.");

    return match fmt_regex_patt.captures(translation_result) {
        Some(matches) => {
            let mut fmt_arg_tokens = quote!{};
            let mut fmt_str = translation_result.to_string();

            for (match_i, each_match) in matches.iter().enumerate() {
                let matched_str = each_match.expect("Regex match is None.").as_str();
                let disclosed_str = &matched_str[1..matched_str.len() - 1];

                let new_arg_ident = if disclosed_str == "" {
                    variant_field_idents.get(match_i).expect(&format!("Format argument `{{{}}}` is invalid.", match_i))
                } else {
                    match disclosed_str.parse::<usize>() {
                        Ok(v) => variant_field_idents.get(v).expect(&format!("Format argument `{{{}}}` is invalid.", v)),
                        Err(_) => disclosed_str,
                    }
                };

                let new_arg_ident = Ident::new(new_arg_ident, Span::call_site());

                let new_arg_token = quote!{
                    #new_arg_ident,
                };

                fmt_arg_tokens.append_all(vec![new_arg_token]);
                fmt_str = fmt_str.replace(matched_str, "{}");
            }

            quote!{
                format!(#fmt_str, #fmt_arg_tokens)
            }
        },
        None => quote!{
            #translation_result
        },
    };
}

#[derive(Clone, Debug, FromDeriveInput)]
#[darling(attributes(translate))]
struct LogTranslationOption {
    data: Data<LogTranslationVariantOption, ()>,
}

#[derive(Clone, Debug, FromVariant)]
#[darling(attributes(translate))]
struct LogTranslationVariantOption {
    ident: Ident,
    fields: darling::ast::Fields<LogTranslationVariantField>,
    #[darling(default)]
    en: Option<String>,
    #[darling(default)]
    ja: Option<String>,
}

#[derive(Clone, Debug, FromField)]
struct LogTranslationVariantField {
    ident: Option<Ident>,
}
