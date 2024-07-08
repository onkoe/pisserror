//! # Variant
//!
//! Helper functions to work around the different enum variant types.

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, Variant};

/// Creates the variant without any placeholders or brackets/parentheses.
///
/// This looks like: `EnumName::VariantName`.
pub(crate) fn make_variant_path(enum_name: &Ident, variant_name: &Ident) -> TokenStream2 {
    quote! {
        #enum_name::#variant_name
    }
}

/// Creates the `match` "head" (path selector with fields) for each variant of
/// an enum.
pub(crate) fn make_match_head(enum_name: &Ident, variant: &Variant) -> TokenStream2 {
    let variant_name = &variant.ident;
    let variant_path = make_variant_path(enum_name, variant_name);

    match &variant.fields {
        syn::Fields::Named(_) => {
            quote! {
                #variant_path{..}
            }
        }
        syn::Fields::Unnamed(fields) => {
            let range = 0..fields.unnamed.len();
            let mapped = range.map(|_| quote!(_,)); // make each value into "_,"
            quote! {#variant_path(#(#mapped)*)} // make a giant "Type::Variant(_, _, _, ...)"
        }
        syn::Fields::Unit => quote! {#variant_path},
    }
}

#[cfg(test)]
mod tests {
    use crate::util::variant::make_match_head;

    #[test]
    fn struct_like_one_field() {
        use quote::quote;
        use syn::{parse_quote, Item};

        let sauce: Item = parse_quote! {
            enum FartsEnum {
                MyVariant {
                    expected: i32,
                    got: i32,
                }
            }
        };

        let Item::Enum(its_an_enum) = sauce else {
            panic!("fuck");
        };

        let enum_name = its_an_enum.ident;
        let variant = its_an_enum.variants.first().unwrap();
        let variant_name = &variant.ident;

        let result = make_match_head(&enum_name, variant);

        assert_eq!(
            result.to_string(),
            quote!(#enum_name::#variant_name{..}).to_string()
        );
    }
}
