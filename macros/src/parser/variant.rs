use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    punctuated::Punctuated, spanned::Spanned as _, Attribute, Ident, Meta, Path, PathSegment,
    Variant,
};

use super::{
    attr::{ErrorAttribute, FromAttribute},
    field::{self, FieldsType, WrappedField, WrappedFieldBuilder, WrappedFields},
};

/// A method to build a `WrappedVariant`.
pub(crate) struct WrappedVariantBuilder {
    variant: Variant,
}

impl WrappedVariantBuilder {
    /// Creates a new `WrappedVariantBuilder` around a given `Variant`.
    pub(crate) const fn new(variant: Variant) -> Self {
        Self { variant }
    }

    /// Attempts to run all build steps to make a `WrappedVariant`.
    pub(crate) fn build(self) -> syn::Result<WrappedVariant> {
        let from_attr_checked = FromAttributeCheck::check_fields(self.variant)?;
        let error_attr_checked = ErrorAttributeCheck::check_errors(from_attr_checked)?;
        Ok(error_attr_checked.finish())
    }
}

/// Step 1: check a variant and its fields
///
/// checks that:
/// - if a variant has a `from` attr, it has no more fields.
pub(crate) struct FromAttributeCheck {
    /// not all variants use a `#[from]` attr
    from_attribute: Option<FromAttribute>,

    // parts of the Variant we were using
    ident: Ident,
    span: Span,
    fields: WrappedFields,
    unparsed_attrs: Vec<Attribute>,
}

impl FromAttributeCheck {
    /// Checks the given variant using the `#[from]` rules.
    pub(crate) fn check_fields(variant: Variant) -> syn::Result<Self> {
        let span = variant.span();
        let Variant {
            fields: vfields,
            attrs: vattrs,
            ident: vident,
            ..
        } = variant;

        let fields_type = match vfields {
            syn::Fields::Named(_) => field::FieldsType::Named,
            syn::Fields::Unnamed(_) => field::FieldsType::Unnamed,
            syn::Fields::Unit => field::FieldsType::Unit,
        };

        // handles the attribute count check internally
        let fields = vfields
            .into_iter()
            .map(move |f| WrappedFieldBuilder::new(f).build())
            .collect::<syn::Result<Vec<_>>>()?;

        let has_from_field = fields.iter().any(WrappedField::has_from_attribute);
        let mut from = None;

        if has_from_field {
            if fields.len() > 1 {
                return Err(Self::err_nonfrom_fields_not_permitted(span));
            }

            // nope, we're clear! let's make the thingy
            if let WrappedField::FromAttribute(from_attr_info) = fields.first().cloned().unwrap() {
                from = Some(from_attr_info);
            } else {
                panic!("we should have a from attr here!!!");
            }
        }

        let wrapped_fields = match fields_type {
            FieldsType::Named => WrappedFields::Named(fields),
            FieldsType::Unnamed => WrappedFields::Unnamed(fields),
            FieldsType::Unit => WrappedFields::Unit,
        };

        Ok(Self {
            ident: vident,
            from_attribute: from,
            fields: wrapped_fields,
            span,
            unparsed_attrs: vattrs,
        })
    }

    /// An error asking users to remove additional fields when using
    /// the from attribute.
    pub(crate) fn err_nonfrom_fields_not_permitted(field_span: Span) -> syn::Error {
        syn::Error::new(
            field_span,
            "A variant containing a field with the `#[from]` attribute must only have one field. \
            Please see: https://github.com/onkoe/pisserror/issues/11#issuecomment-2215435824",
        )
    }
}

/// Step 2: check variant for an error tag
///
/// checks that:
/// - variant has an error tag.
/// - the error tag should either:
///    - have a string, or be
///    - #[error(transparent)] for a from variant
pub(crate) struct ErrorAttributeCheck {
    /// not all variants use a `#[from]` attr
    ident: Ident,
    fields: WrappedFields,
    from_attribute: Option<FromAttribute>,
    error_attribute: ErrorAttribute,
}

impl ErrorAttributeCheck {
    pub(crate) fn check_errors(variant: FromAttributeCheck) -> syn::Result<Self> {
        let FromAttributeCheck {
            from_attribute,
            ident,
            span,
            fields,
            unparsed_attrs: attrs,
        } = variant;

        let error_attribute_path = crate::util::create_path(span, &["error"]);

        let mut error_attributes = attrs.iter().filter(|a| a.path() == &error_attribute_path);

        // warning: this mutates error_attributes (the iterator is being consumed)
        let (first, second) = (error_attributes.next(), error_attributes.next());

        // check if we got any problems. otherwise, grab the metalist for f-string
        let error_attribute = match (first, second) {
            (None, _) => {
                return Err(Self::err_missing_error_attr(span));
            }
            (Some(_), Some(second_err_attr)) => {
                return Err(Self::err_multiple_error_attrs(second_err_attr));
            }
            (Some(attr), None) => {
                let Meta::List(ref attr_args) = attr.meta else {
                    // give a more specific error, depending on if we have a `#[from]` field
                    return Err(if from_attribute.is_some() {
                        Self::err_nothing_to_display_from(attr)
                    } else {
                        Self::err_nothing_to_display(attr)
                    });
                };

                // make sure the attribute has something inside it
                if attr_args.tokens.is_empty() {
                    return Err(Self::err_nothing_to_display(attr));
                }

                let transparent_check =
                    { attr_args.tokens.to_string() == ErrorAttribute::TRANSPARENT_LITERAL };

                // let transparent_attr_path =
                //     &util::create_path(span, &[ErrorAttribute::TRANSPARENT_LITERAL]);

                // check if we're stringy or just have `transparent`
                if transparent_check {
                    // ok now make sure we have a `#[from]` attr
                    if from_attribute.is_none() {
                        return Err(Self::err_transparent_requires_from_variant(attr));
                    }

                    ErrorAttribute::Transparent
                } else {
                    ErrorAttribute::Stringy(attr_args.tokens.clone())
                }
            }
        };

        Ok(Self {
            ident,
            fields,
            from_attribute,
            error_attribute,
        })
    }

    /// Since this is the last step, this creates the `WrappedVariant`.
    pub(crate) fn finish(self) -> WrappedVariant {
        WrappedVariant {
            ident: self.ident,
            fields: self.fields,
            from_attribute: self.from_attribute,
            error_attribute: self.error_attribute,
        }
    }

    fn err_missing_error_attr(span: Span) -> syn::Error {
        syn::Error::new(
            span,
            "Each variant must have a corresponding `#[error(...)]` attribute.",
        )
    }

    fn err_multiple_error_attrs(second_attr: &Attribute) -> syn::Error {
        syn::Error::new_spanned(
            second_attr,
            "Each variant may only have one `#[error(...)]` attribute.",
        )
    }

    fn err_nothing_to_display(attr: &Attribute) -> syn::Error {
        syn::Error::new_spanned(
            attr,
            "An `#[error(...)]` attribute must contain a format_args!() \
            f-string to implement `Display`.",
        )
    }

    fn err_nothing_to_display_from(attr: &Attribute) -> syn::Error {
        syn::Error::new_spanned(
            attr,
            "A `#[from]` variant's `#[error(...)]` attribute must contain \
            `transparent` or a valid format_args!() f-string.",
        )
    }

    fn err_transparent_requires_from_variant(attr: &Attribute) -> syn::Error {
        syn::Error::new_spanned(
            attr,
            "An `#[error(transparent)]` attribute requires a field marked with \
            `#[from]`.",
        )
    }
}

#[derive(Debug)]
pub(crate) struct WrappedVariant {
    pub ident: Ident,
    pub fields: WrappedFields,
    pub from_attribute: Option<FromAttribute>,
    pub error_attribute: ErrorAttribute,
}

impl WrappedVariant {
    /// Creates the path for this variant given the enum identifer.
    ///
    /// Paths look like: `EnumName::Variant`, with no extras.
    pub(crate) fn variant_path(&self, enum_ident: Ident) -> Path {
        Path {
            leading_colon: None,
            segments: {
                let mut p = Punctuated::new();
                p.push(PathSegment::from(enum_ident));
                p.push(PathSegment::from(self.ident.clone()));
                p
            },
        }
    }

    /// Makes the head of a match arm. That is: `ThisPart(..) => { ... }`
    pub(crate) fn match_head(&self, enum_ident: Ident) -> TokenStream2 {
        let variant_path = self.variant_path(enum_ident);

        match self.fields {
            WrappedFields::Named(_) => quote! {#variant_path{..}},
            WrappedFields::Unnamed(_) => quote! {#variant_path(..)},
            WrappedFields::Unit => quote! {#variant_path},
        }
    }

    /// A match head that's filled with identifiers. For example:
    /// `SomeEnum::SomeVariant::(_0, _1, _2)`
    pub(crate) fn filled_match_head(&self, enum_ident: Ident) -> TokenStream2 {
        let variant_path = self.variant_path(enum_ident);

        match self.fields {
            WrappedFields::Named(ref n) => {
                let list = n.iter().map(|field| {
                    let name = match *field {
                        WrappedField::Typical(ref info) | WrappedField::FromAttribute(ref info) => {
                            &info.ident
                        }
                    };
                    quote!(ref #name)
                });

                quote! {
                    #variant_path{#(#list), *}
                }
            }

            WrappedFields::Unnamed(ref un) => {
                let field_range = (0..un.len()).map(|i| {
                    let ident = quote::format_ident!("_{}", i);
                    quote!(ref #ident)
                });

                // FIXME(#14): users currently have to do `_0` which is... bad
                quote! {
                    #variant_path(#(#field_range), *)
                }
            }

            WrappedFields::Unit => quote! {#variant_path},
        }
    }
}

/**
To implement `Display`, we need to parse the given error message for each
variant.

However, there needs to be one error attribute per - not more, not less.

I've made some tests below verifying this assumption.

First, having no `#[error]` attribute should fail:
```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
#[allow(unused)]
enum MyError {
    VariantOne,
}
```

Also, you can't have too many of them, either:

```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
#[allow(unused)]
enum MyError {
    #[error("first attr")]
    #[error("second attr")]
    VariantOne,
}
``` */
#[allow(unused)]
struct DoctestErrorAttr;

/// Parses the user's enum's variants to check for any internal `#[from]`
/// attributes, then generates code that matches on any given error variant.
///
/// # Attribute Rules
///
/// The `#[from]` attribute assumes it's only used once per variant.
///
/// As such, the following code shouldn't compile:
///
/** ```compile_fail
use macros::Error;
use std::error::Error;

#[derive(Debug, Error)]
enum SomeError {
    // you can't have two `#[from]` attrs on one variant!
    #[error("hello")]
    TwoAttrsOneField(#[from] std::io::Error, #[from] std::fmt::Error),
}
``` */
#[allow(unused)]
struct DoctestFromAttr;

#[cfg(test)]
mod tests {
    use crate::parser::{field, variant, UserEnum};
    use syn::{parse_quote, ItemEnum};

    #[test]
    fn struct_like_one_field() {
        use quote::quote;
        use syn::{parse_quote, Item};

        let sauce: Item = parse_quote! {
            enum FartsEnum {
                #[error("mfw the test needs a message 😧")]
                MyVariant {
                    expected: i32,
                    got: i32,
                }
            }
        };

        let Item::Enum(its_an_enum) = sauce else {
            panic!("fuck");
        };

        let user_enum = UserEnum::new(its_an_enum.into()).unwrap();
        let v = user_enum.variants.first().unwrap();

        let enum_ident = user_enum.ident();
        let variant_ident = v.ident.clone();

        let result = v.match_head(enum_ident.clone());

        assert_eq!(
            result.to_string(),
            quote!(#enum_ident::#variant_ident{..}).to_string()
        );
    }

    #[test]
    #[allow(clippy::indexing_slicing)]
    fn parses_from_variants_correctly() {
        let sauce: ItemEnum = parse_quote! {
            enum Piss {
                #[error("💦 💛")]
                FromVariantOne(#[from] std::io::Error),
                #[error("r# don't break please r#\n\r# r# 3###\n# ## ##")]
                FromVariantTwo {
                    #[from]
                    from: std::collections::TryReserveError,
                },
                #[error("laaaame!")]
                LameVariant,
            }
        };

        // check if the fieldbuilder is fucking up
        let v2_results = field::WrappedFieldBuilder::new(
            sauce.variants[1].fields.iter().next().unwrap().clone(),
        )
        .build()
        .unwrap();

        match v2_results {
            field::WrappedField::Typical(ref info) => panic!("{:?}", info.ident),
            field::WrappedField::FromAttribute(_) => (),
        };

        assert!(v2_results.has_from_attribute());

        assert_eq!(
            sauce
                .clone()
                .variants
                .into_iter()
                .map(|v| variant::FromAttributeCheck::check_fields(v)
                    .unwrap()
                    .from_attribute
                    .is_some())
                .collect::<Vec<bool>>(),
            vec![true, true, false]
        );

        let user_enum = UserEnum::new(sauce.into()).unwrap();

        let expected = vec![true, true, false];
        let got: Vec<bool> = user_enum
            .variants
            .iter()
            .map(|v| v.from_attribute.is_some())
            .collect();

        assert_eq!(expected, got);
    }
}
