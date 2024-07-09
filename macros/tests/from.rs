use macros::Error;
use std::error::Error;

// This one only has one attr, so it works fine.
#[derive(Debug, Error)]
enum SomeTupleEnumError {
    #[error("hello tuple enum")]
    FromIo(#[from] std::io::Error),
}

#[derive(Debug, Error)]
enum SomeStructEnumError {
    #[error("hello struct enum")]
    FromIo {
        #[from]
        from: std::io::Error,
    },
}

#[test]
fn t() {
    let ioerr = std::io::Error::from_raw_os_error(22);
    let _some_struct_err = SomeStructEnumError::from(ioerr);

    // TODO: try formatting the error
}
