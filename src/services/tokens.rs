pub fn estimate_tokens(text: &str) -> u64 {
    if text.is_empty() {
        return 0;
    }

    let byte_count = text.len() as u64;
    let word_count = text.split_ascii_whitespace().count() as u64;

    assert!(byte_count >= word_count);

    let estimate_bytes = byte_count.div_euclid(4);
    let estimate_words = (word_count * 13).div_euclid(10);
    let estimate = u64::midpoint(estimate_bytes, estimate_words);

    assert!(estimate <= byte_count);

    estimate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_has_no_tokens() {
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn estimate_grows_with_length() {
        let short = estimate_tokens("fn main() {}");
        let long = estimate_tokens("fn main() {} fn other() {} fn third() {}");

        assert!(long > short);
    }
}
