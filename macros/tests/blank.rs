#[cfg(test)]
mod tests {
    use pisserror_macros::Error;
    use std::error::Error;

    /// An error type that has no variants (completely valid)
    #[derive(Debug, Error)]
    #[allow(unused)]
    enum BlankError {}
}
