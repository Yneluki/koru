#[derive(Clone, Debug)]
pub struct GroupName(String);

impl TryFrom<String> for GroupName {
    type Error = &'static str;

    fn try_from(n: String) -> Result<Self, Self::Error> {
        if n.is_empty() {
            Err("Group Name cannot be empty")
        } else {
            Ok(Self(n))
        }
    }
}

impl From<GroupName> for String {
    fn from(n: GroupName) -> Self {
        n.0
    }
}
