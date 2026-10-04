const IDENTIFIER_BYTES_MAX: usize = 64;

pub fn generate() -> String {
    let identifier = uuid::Uuid::new_v4().to_string();

    assert!(is_valid(&identifier));

    identifier
}

pub fn is_valid(identifier: &str) -> bool {
    if identifier.is_empty() {
        return false;
    }

    if identifier.len() > IDENTIFIER_BYTES_MAX {
        return false;
    }

    identifier
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_identifiers_are_valid_and_distinct() {
        let first = generate();
        let second = generate();

        assert!(is_valid(&first));
        assert_ne!(first, second);
    }

    #[test]
    fn path_shaped_identifiers_are_rejected() {
        assert!(!is_valid(""));
        assert!(!is_valid("../../escape"));
        assert!(!is_valid("a/b"));
        assert!(!is_valid("a\\b"));
        assert!(!is_valid("name.json"));
        assert!(!is_valid(&"a".repeat(IDENTIFIER_BYTES_MAX + 1)));
    }
}
