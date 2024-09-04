//! # Error
//!
//! Implements the `Error` trait for the user's error type.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser::{field::WrappedFields, UserEnum};

impl UserEnum {
    /// The `Error` trait's `source` method.
    pub(crate) fn source(&self) -> TokenStream2 {
        let match_arms = self.variants().iter().map(|v| {
            if let Some(ref info) = v.from_attribute {
                let variant_path = v.variant_path(self.ident());

                match v.fields {
                    WrappedFields::Named(_) => {
                        let from_ident = info.ident.clone().unwrap();
                        quote! { #variant_path { ref #from_ident } => Some(#from_ident)}
                        // named_arm
                    }
                    WrappedFields::Unnamed(_) => quote! { #variant_path(ref e) => Some(e) },
                    WrappedFields::Unit => {
                        unreachable!("unit enums cannot have a #[from] field")
                    }
                }
            } else {
                let left_side = v.match_head(self.ident());
                quote! {#left_side => None}
            }
        });

        quote! {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match *self {
                    #(#match_arms),*
                }
            }
        }
    }

    /// The `Error` trait's `descritpion` method.
    ///
    /// The method this generates is deprecated in favor of `Display`/`ToString`
    /// on Error types, so we can safely return an empty string slice.
    pub(crate) fn description() -> TokenStream2 {
        // TODO: consider using `Display` instead? check with other libraries b4.

        quote! {
            fn description(&self) -> &str {
                &""
            }
        }
    }

    /// The `Error` trait's `cause` method. Now deprecated in favor of
    /// `source`, which has the 'static bound.
    ///
    /// As such, this code always returns None.
    pub(crate) fn cause() -> TokenStream2 {
        quote! {
            fn cause(&self) -> Option<&dyn Error> {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream as TokenStream2;
    use syn::{parse_quote, ItemEnum};

    use crate::parser::UserEnum;

    #[test]
    fn source_should_use_from() {
        let sauce: ItemEnum = parse_quote! {
            enum Piss {
                #[error("💦 💛")]
                FromVariantOne(#[from] std::io::Error),
                #[error("r# don't break please r#\n\r# r# 3###\n# ## ##")]
                FromVariantTwo {
                    #[from]
                    any_name_you_want: std::collections::TryReserveError,
                },
                #[error("laaaame!")]
                LameVariant,
            }
        };
        let user_enum = UserEnum::new(sauce.into()).unwrap();

        let expected: TokenStream2 = parse_quote! {
            fn source(&self) -> Option<&(dyn Error + 'static)> {
                match *self {
                    Piss::FromVariantOne(ref e) => Some(e),
                    Piss::FromVariantTwo { ref any_name_you_want } => Some(any_name_you_want),
                    Piss::LameVariant => None
                }
            }
        };
        let got = user_enum.source();

        assert_eq!(expected.to_string(), got.to_string());
    }

    #[test]
    fn description_shouldnt_change() {
        let expected: TokenStream2 = parse_quote! {
            fn description(&self) -> &str {
                &""
            }
        };
        let got = UserEnum::description();

        assert_eq!(expected.to_string(), got.to_string());
    }

    #[test]
    fn cause_shouldnt_change() {
        let expected: TokenStream2 = parse_quote! {
            fn cause(&self) -> Option<&dyn Error> {
                None
            }
        };
        let got = UserEnum::cause();

        assert_eq!(expected.to_string(), got.to_string());
    }
}
