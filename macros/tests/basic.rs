#[cfg(test)]
#[allow(clippy::print_stderr)]
#[allow(clippy::use_debug)]
mod tests {
    use pisserror_macros::Error;
    use std::error::Error;

    #[derive(Debug, Error)]
    enum MyErrorType {
        #[error("1 {}", "am i a genius?")]
        Thing1,
        #[error("2")]
        #[allow(dead_code)]
        Thing2,
    }

    #[test]
    fn farts() -> Result<(), MyErrorType> {
        let error: Result<i32, MyErrorType> = Err(MyErrorType::Thing1);

        eprintln!("{error:?}");
        Ok(())
    }
}
