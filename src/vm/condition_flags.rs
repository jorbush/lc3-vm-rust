#[derive(Debug, Clone, Copy)]
pub enum ConditionFlag {
    Pos = 1 << 0, /* P */
    Zro = 1 << 1, /* Z */
    Neg = 1 << 2, /* N */
}

impl From<ConditionFlag> for u16 {
    fn from(val: ConditionFlag) -> Self {
        val as u16
    }
}

impl TryFrom<u16> for ConditionFlag {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::Pos,
            2 => Self::Zro,
            4 => Self::Neg,
            _ => return Err("invalid condition flag"),
        })
    }
}
