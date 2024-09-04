#[cfg(test)]
mod tests {
    use core::error::Error;
    use pisserror_macros::Error;

    /// An error type that has no variants (completely valid)
    #[derive(Debug, Error)]
    #[expect(unused, reason = "compile test")]
    enum BlankError {}
}
