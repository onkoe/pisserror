use macros::Error;
use std::error::Error;

// This one only has one attr, so it works fine.
#[derive(Debug, Error)]
enum SomeError {
    #[error("hello")]
    OneAttrOneField(#[from] std::io::Error),
}
