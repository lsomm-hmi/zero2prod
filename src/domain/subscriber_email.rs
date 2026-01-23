use validator::{Validate, ValidationErrors};

#[derive(Clone, Debug, Validate)]
pub struct SubscriberEmail {
    #[validate(email)]
    address: String,
}

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, ValidationErrors> {
        let email = Self { address: s };
        email.validate()?;
        Ok(email)
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.address
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claims::{assert_err, assert_ok};
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use proptest::prelude::*;

    #[test]
    fn empty_string_is_reject() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    proptest! {
        #[test]
        fn valid_emails_are_parsed_successfully(email in safe_email_strategy()) {
            dbg!(&email);
            assert_ok!(SubscriberEmail::parse(email));
        }
    }

    fn safe_email_strategy() -> impl Strategy<Value = String> {
        // Use a fixed seed for reproducibility, or use thread_rng()
        Just(()).prop_perturb(|_, mut rng| SafeEmail().fake_with_rng(&mut rng))
    }
}
