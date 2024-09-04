//! # From
//!
//! Implements the `From` trait for the user's error type.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser::{field::WrappedFields, UserEnum};

impl UserEnum {
    /// Returns ALL `From` implementations for the `#[from]` variants of the
    /// user's enum.
    pub(crate) fn from(&self) -> TokenStream2 {
        let enum_ident = self.ident();

        // create a new `From<external::Error> for UserError` for each from variant
        let from_impls = self
            .variants()
            .iter()
            .filter(|v| v.from_attribute.is_some())
            .map(|from_v| {
                let variant_ident = from_v.ident.clone();
                let from_attr = from_v.from_attribute.clone().unwrap();
                let from_type = from_attr.ty;

                // let's decide which style to use during construction
                let style = match from_v.fields {
                    WrappedFields::Named(_) => {
                        let from_ident = from_attr.ident.unwrap();
                        quote!(#enum_ident::#variant_ident {#from_ident: value})
                    }
                    WrappedFields::Unnamed(_) => {
                        quote!(#enum_ident::#variant_ident(value))
                    }
                    WrappedFields::Unit => unreachable!(),
                };

                quote! {
                    #[automatically_derived]
                    impl core::convert::From<#from_type> for #enum_ident {
                        fn from(value: #from_type) -> Self {
                            #style
                        }
                    }
                }
            });

        quote! {
            #(#from_impls)*
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{variant, UserEnum};
    use proc_macro2::TokenStream as TokenStream2;
    use syn::{parse_quote, spanned::Spanned as _, ItemEnum};

    #[test]
    fn error_on_multiple_froms() {
        // make the enum
        let sauce: ItemEnum = parse_quote! {
            enum MyError {
                #[error("some variant? uh oh")]
                // you may not have multiple from attributes, so this should fail
                SomeVariant(#[from] std::io::Error, #[from] std::collections::TryReserveError)
            }
        };

        let span = sauce.span();

        let user_enum = UserEnum::new(sauce.into());
        assert_eq!(
            user_enum.err().unwrap().to_string(),
            variant::FromAttributeCheck::err_nonfrom_fields_not_permitted(span).to_string()
        );
    }

    #[test]
    fn works_with_many_variants() {
        // make the enum
        let sauce: ItemEnum = parse_quote! {
            // these variants all have one from - it should work great!
            enum MyError {
                #[error("one")]
                VariantOne(#[from] std::io::Error),
                #[error("two")]
                VariantTwo(#[from] std::collections::TryReserveError),
                #[error("three")]
                VariantThree {
                    #[from]
                    some_field: std::array::TryFromSliceError,
                }
            }
        };

        let user_enum = UserEnum::new(sauce.into()).unwrap();

        let expected: TokenStream2 = parse_quote! {
            #[automatically_derived]
            impl core::convert::From<std::io::Error> for MyError {
                fn from(value: std::io::Error) -> Self {
                    MyError::VariantOne(value)
                }
            }
            #[automatically_derived]
            impl core::convert::From<std::collections::TryReserveError> for MyError {
                fn from(value: std::collections::TryReserveError) -> Self {
                    MyError::VariantTwo(value)
                }
            }
            #[automatically_derived]
            impl core::convert::From<std::array::TryFromSliceError> for MyError {
                fn from(value: std::array::TryFromSliceError) -> Self {
                    MyError::VariantThree {
                        some_field: value
                    }
                }
            }
        };

        // check it
        assert_eq!(user_enum.from().to_string(), expected.to_string());
    }

    #[test]
    fn errs_with_multiple_fields() {
        // make the enum
        let sauce: ItemEnum = parse_quote! {
            enum MyError {
                #[error("struct-like variant")]
                // if a variant's field has a `#[from]` attr, it MUST be the only field
                StructLikeVariant {
                    #[from]
                    some_error_type: std::io::Error,
                    favorite_number: u32,
                }
            }
        };

        let span = sauce.span();
        let user_enum = UserEnum::new(sauce.into());

        assert_eq!(
            user_enum.err().unwrap().to_string(),
            variant::FromAttributeCheck::err_nonfrom_fields_not_permitted(span).to_string()
        );
    }
}
