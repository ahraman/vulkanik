use proc_macro2::TokenStream;
use quote::quote;

use super::parse::{ParsedEnum, ParsedVariant};

impl ParsedEnum {
    pub fn expand(self) -> TokenStream {
        self.expand_from_attr_impl()
    }

    fn expand_from_attr_impl(&self) -> TokenStream {
        let from_attr_fn = self.expand_from_attr_fn();

        let name = &self.name;
        quote! {
            impl FromAttr for #name {
                #from_attr_fn
            }
        }
    }

    fn expand_from_attr_fn(&self) -> TokenStream {
        let match_cases = self.expand_from_attr_fn_match_cases();
        quote! {
            fn from_attr(value: Option<String>) -> Result<Self, Option<String>> {
                match value {
                    Some(value) => Ok(match value.as_str() {
                        #match_cases
                        _ => return Err(Some(value)),
                    }),
                    None => Err(None),
                }
            }
        }
    }

    fn expand_from_attr_fn_match_cases(&self) -> TokenStream {
        let mut match_cases = TokenStream::new();

        for variant in &self.variants {
            let match_case = variant.expand_from_attr_fn_match_case();
            match_cases = quote! {
                #match_cases
                #match_case
            }
        }

        match_cases
    }
}

impl ParsedVariant {
    fn expand_from_attr_fn_match_case(&self) -> TokenStream {
        let name = &self.name;
        let name_str = &self.name_str;
        quote! {
            #name_str => Self::#name,
        }
    }
}
