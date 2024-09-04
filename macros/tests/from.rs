#[cfg(test)]
mod tests {
    use core::{error::Error, num::ParseIntError, str::ParseBoolError};
    use pisserror_macros::Error;

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

    #[derive(Debug, Error)]
    enum Transparent {
        #[error(transparent)]
        Structlike {
            #[from]
            err_is_from: ParseIntError,
        },
        #[error(transparent)]
        Tuplelike(#[from] ParseBoolError),
    }
}
