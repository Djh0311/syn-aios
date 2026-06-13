pub(crate) fn normalize_slash_lowercase(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_slash_lowercase_trims_slashes_and_lowercases() {
        assert_eq!(
            normalize_slash_lowercase("  Foo\\Bar\\BAZ  "),
            "foo/bar/baz"
        );
    }

    #[test]
    fn normalize_slash_lowercase_preserves_inner_whitespace() {
        assert_eq!(normalize_slash_lowercase(" A  B "), "a  b");
    }
}
