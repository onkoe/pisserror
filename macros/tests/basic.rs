#[cfg(test)]
#[expect(clippy::print_stderr, reason = "to use the error")]
#[expect(clippy::use_debug, reason = "to read the error")]
mod tests {
    use core::error::Error;
    use pisserror_macros::Error;

    #[derive(Debug, Error)]
    enum MyErrorType {
        #[error("1 {}", "am i a genius?")]
        Thing1,
        #[error("2")]
        #[expect(dead_code, reason = "just an extra variant")]
        Thing2,
    }

    #[test]
    fn farts() -> Result<(), MyErrorType> {
        let error: Result<i32, MyErrorType> = Err(MyErrorType::Thing1);

        eprintln!("{error:?}");
        Ok(())
    }
}
