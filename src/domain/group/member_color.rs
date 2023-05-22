#[derive(Debug, Clone)]
pub struct MemberColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl TryFrom<String> for MemberColor {
    type Error = &'static str;

    fn try_from(n: String) -> Result<Self, Self::Error> {
        if n.is_empty() {
            Err("Color cannot be empty")
        } else {
            let parts: Vec<&str> = n.split(',').collect();
            if parts.len() != 3 {
                return Err("Color should only have 3 parts (r,g,b)");
            }
            Ok(Self {
                red: parts[0]
                    .to_string()
                    .parse::<u8>()
                    .map_err(|_| "Color should be between 0 and 255")?,
                green: parts[1]
                    .to_string()
                    .parse::<u8>()
                    .map_err(|_| "Color should be between 0 and 255")?,
                blue: parts[2]
                    .to_string()
                    .parse::<u8>()
                    .map_err(|_| "Color should be between 0 and 255")?,
            })
        }
    }
}

impl From<MemberColor> for String {
    fn from(n: MemberColor) -> Self {
        format!("{},{},{}", n.red, n.green, n.blue)
    }
}

impl Default for MemberColor {
    fn default() -> Self {
        MemberColor {
            red: 0,
            green: 255,
            blue: 0,
        }
    }
}
