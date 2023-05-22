#[derive(Clone, Debug)]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = &'static str;

    fn try_from(n: String) -> Result<Self, Self::Error> {
        if n.is_empty() {
            Err("Email cannot be empty")
        } else if !n.contains('@') {
            Err("Email seems invalid")
        } else {
            Ok(Self(n))
        }
    }
}

impl From<Email> for String {
    fn from(n: Email) -> Self {
        n.0
    }
}

impl Default for Email {
    fn default() -> Self {
        Email("unknown@unknown.com".to_string())
    }
}
