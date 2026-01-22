use validator::{Validate, ValidationErrors};

#[derive(Debug, Validate)]
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
    use claims::assert_err;

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
}
