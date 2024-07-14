use std::marker::PhantomData;

use proc_macro2::Span;
use syn::{spanned::Spanned as _, Attribute, Field, Ident, Type};

use crate::util;

/// Something like `syn::Fields`, but all the fields have been checked.
pub enum WrappedFields {
    Named(Vec<WrappedField>),
    Unnamed(Vec<WrappedField>),
    Unit,
}

/// `syn::Fields`, minus the fields. This is here to avoid some nasty logic.
pub enum FieldsType {
    Named,
    Unnamed,
    Unit,
}

pub trait FieldBuildingStep {}

pub struct CreationStep {}
pub struct FromScanStep {}

impl FieldBuildingStep for CreationStep {}
impl FieldBuildingStep for FromScanStep {}

pub struct WrappedFieldBuilder<T: FieldBuildingStep> {
    field: Field,
    _step: PhantomData<T>,
}

impl<T: FieldBuildingStep> WrappedFieldBuilder<T> {
    /// Takes the info and attributes from a given field, consuming the field.
    pub fn field_info_and_attrs(field: Field) -> (WrappedFieldInfo, Vec<Attribute>) {
        let span = field.span();

        let Field {
            ident: name,
            ty,
            attrs,
            ..
        } = field;

        (WrappedFieldInfo { name, ty, span }, attrs)
    }

    pub fn err_too_many_from_attributes(field_span: &Span) -> syn::Error {
        syn::Error::new(
            *field_span,
            "You may only have one `#[from]` attribute per variant.",
        )
    }
}

impl WrappedFieldBuilder<FromScanStep> {
    /// Checks this field for a `from` attribute. Returns an error if the
    /// `from` rules are violated.
    ///
    /// Otherwise, you'll get a `WrappedField`.
    pub fn scan_for_froms(self) -> syn::Result<WrappedField> {
        let (field_info, attrs) = Self::field_info_and_attrs(self.field);
        let field_span = &field_info.span;
        let from_attribute_path = &util::create_path(field_info.span, &["from"]);

        let mut already_found_from_attribute = false;

        for attr in attrs {
            if attr.path() == from_attribute_path {
                if already_found_from_attribute {
                    return Err(Self::err_too_many_from_attributes(field_span));
                }

                already_found_from_attribute = true;
            }
        }

        Ok(match already_found_from_attribute {
            true => WrappedField::Typical(field_info),
            false => WrappedField::Typical(field_info),
        })
    }
}

impl WrappedFieldBuilder<CreationStep> {
    pub fn new(field: Field) -> WrappedFieldBuilder<FromScanStep> {
        WrappedFieldBuilder {
            field,
            _step: PhantomData,
        }
    }
}

/// Some information about a field.
#[derive(Clone)]
pub struct WrappedFieldInfo {
    pub name: Option<Ident>,
    pub ty: Type,
    pub span: Span,
}

/// A field.
#[derive(Clone)]
pub enum WrappedField {
    Typical(WrappedFieldInfo),
    FromAttribute(WrappedFieldInfo),
}

impl WrappedField {
    /// Checks if this field has the `#[from]` attribute.
    pub fn has_from_attribute(&self) -> bool {
        match self {
            Self::FromAttribute(_) => true,
            Self::Typical(_) => false,
        }
    }

    // TODO: remove whichever one of these i dont use

    pub fn info(&self) -> &WrappedFieldInfo {
        match self {
            WrappedField::Typical(info) | WrappedField::FromAttribute(info) => info,
        }
    }

    pub fn into_info(self) -> WrappedFieldInfo {
        match self {
            WrappedField::Typical(info) | WrappedField::FromAttribute(info) => info,
        }
    }
}
