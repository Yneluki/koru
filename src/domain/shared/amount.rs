#[derive(Copy, Clone, Debug)]
pub struct Amount(f32);

impl TryFrom<f32> for Amount {
    type Error = &'static str;

    fn try_from(n: f32) -> Result<Self, Self::Error> {
        if n <= 0.0 {
            Err("Amount should be more than 0")
        } else {
            Ok(Self(n))
        }
    }
}

impl From<Amount> for f32 {
    fn from(n: Amount) -> Self {
        n.0
    }
}
