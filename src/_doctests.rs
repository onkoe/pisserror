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
