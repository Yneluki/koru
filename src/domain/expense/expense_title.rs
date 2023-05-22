#[derive(Clone, Debug)]
pub struct ExpenseTitle(String);

impl TryFrom<String> for ExpenseTitle {
    type Error = &'static str;

    fn try_from(n: String) -> Result<Self, Self::Error> {
        if n.is_empty() {
            Err("Expense Description cannot be empty")
        } else {
            Ok(Self(n))
        }
    }
}

impl From<ExpenseTitle> for String {
    fn from(n: ExpenseTitle) -> Self {
        n.0
    }
}
