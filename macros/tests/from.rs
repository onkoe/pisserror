use macros::Error;
use std::error::Error;

// This one only has one attr, so it works fine.
#[derive(Debug, Error)]
enum SomeTupleEnumError {
    #[error("hello tuple enum")]
    OneAttrOneField(#[from] std::io::Error),
}

#[derive(Debug, Error)]
enum SomeStructEnumError {
    #[error("hello struct enum")]
    FromIo {
        #[from]
        from: std::io::Error,
    },
}
