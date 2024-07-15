//! # From
//!
//! Implements the `From` trait for the user's error type.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::parser::{field::WrappedFields, UserEnum};

impl UserEnum {
    /// Returns ALL `From` implementations for the `#[from]` variants of the
    /// user's enum.
    pub fn from(&self) -> TokenStream2 {
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
                let style = match &from_v.fields {
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
    mod field_checking_tests {
        use crate::parser::UserEnum;
        use syn::{parse_quote, ItemEnum};

        #[test]
        fn error_on_multiple_froms() {
            // make the enum
            let sauce: ItemEnum = parse_quote! {
                enum MyError {
                    #[error("some variant? uh oh")]
                    // you may not have multiple from attributes, so this should fail
                    SomeVariant(#[from] std::io::Error, #[from] std::collection::TryReserveError)
                }
            };

            let user_enum = UserEnum::new(sauce.into());
            assert!(user_enum.is_err());
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
                    VariantTwo(#[from] std::collection::TryReserveError),
                    #[error("three")]
                    VariantThree {
                        #[from]
                        some_field: std::array::TryFromSliceError,
                    }
                }
            };

            let user_enum = UserEnum::new(sauce.into()).unwrap();

            // check it
            let x = user_enum.from();
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

            let user_enum = UserEnum::new(sauce.into());
            assert!(user_enum.is_err());
        }
    }

    mod from_trait_gen_tests {
        #[test]
        fn two_variants_one_error() {
            // two variants with the same From<Error> should fail
            // assert!(result.is_err());
        }
    }
}
