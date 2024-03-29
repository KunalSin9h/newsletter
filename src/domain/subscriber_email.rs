use validator::validate_email;

#[derive(Debug, Clone)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<SubscriberEmail, String> {
        if validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email", s))
        }
    }
}

impl std::fmt::Display for SubscriberEmail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for SubscriberEmail {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::{assert_err, assert_ok};

    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[test]
    fn empty_email_is_rejected() {
        let a = "".to_string();
        assert_err!(SubscriberEmail::parse(a));
    }

    #[test]
    fn missing_email_symbol_is_rejected() {
        let a = "domain.com".to_string();
        assert_err!(SubscriberEmail::parse(a));
    }

    #[test]
    fn missing_email_subject_is_rejected() {
        let a = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(a));
    }

    #[test]
    fn valid_email_is_passes_successfully() {
        // TODO
        // use quickcheck or proptest for better property testing
        for _ in 0..100 {
            let a = SafeEmail().fake();
            assert_ok!(SubscriberEmail::parse(a));
        }
    }
}
