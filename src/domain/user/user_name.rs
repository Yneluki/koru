#[derive(Clone, Debug)]
pub struct UserName(String);

impl TryFrom<String> for UserName {
    type Error = &'static str;

    fn try_from(n: String) -> Result<Self, Self::Error> {
        if n.is_empty() {
            Err("User Name cannot be empty")
        } else {
            Ok(Self(n))
        }
    }
}

impl From<UserName> for String {
    fn from(n: UserName) -> Self {
        n.0
    }
}

impl Default for UserName {
    fn default() -> Self {
        UserName("Unknown".to_string())
    }
}
