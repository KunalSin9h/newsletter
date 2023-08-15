use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(input: String) -> Result<SubscriberName, String> {
        let is_empty_input = input.trim().is_empty();

        let is_too_long = input.graphemes(true).count() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        let has_forbidden_char = input.chars().any(|val| forbidden_characters.contains(&val));

        if is_empty_input || is_too_long || has_forbidden_char {
            return Err(format!("{} is not a valid subscriber name", input));
        }

        Ok(Self(input))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let a = "a".repeat(256);
        assert_ok!(SubscriberName::parse(a));
    }

    #[test]
    fn a_name_longer_then_256_is_rejected() {
        let a = "a".repeat(257);
        assert_err!(SubscriberName::parse(a));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let a = " ".repeat(256);
        assert_err!(SubscriberName::parse(a));
    }

    #[test]
    fn empty_string_is_rejected() {
        let a = "".to_string();
        assert_err!(SubscriberName::parse(a));
    }

    #[test]
    fn name_containing_a_invalid_char_is_rejected() {
        for char in &['/', '(', ')', '<', '>', '\\', '{', '}'] {
            let name = char.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_successful() {
        let name = "Kunal Singh".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}
