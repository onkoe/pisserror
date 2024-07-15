use proc_macro2::TokenStream;

use super::field::WrappedFieldInfo;

// #[from] contains a name and type. but that's just a field!
pub type FromAttribute = WrappedFieldInfo;

pub struct ErrorAttribute {
    // TODO(#15): `Option<TokenStream>` if we allow `from` variants to go without a message?
    pub format_string: TokenStream,
}
