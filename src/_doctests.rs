/**
```compile_fail
use super::Error;
use std::error::Error;

#[derive(Debug, Error)]
enum FromAttrWithArgs {
    #[error(transparent)]
    ErrorVariant(#[from("this shouldn't work")] std::io::Error),
}
```
*/
pub fn from_attr_args_shouldnt_have_args() {}

/**
The following test should fail to compile, as `Error::source` returns has a
'static bound on its return value.

As such, you can't use non-static lifetimes on an Error properly implementing
the `source` method.

```compile_fail
use core::error::Error;
use pisserror_macros::Error;

#[derive(Clone, Debug, Error)]
enum ErrorTyWeWillBorrowTickA {
    #[error("")]
    Variant,
}

#[derive(Clone, Debug, Error)]
enum MyError<'a> {
    #[error("from err with non-static lifetime. this won't compile.")]
    FromRefLt(#[from] &'a ErrorTyWeWillBorrowTickA),
}
```
*/
pub fn from_attr_cant_use_nonstatic_lifetime() {}
