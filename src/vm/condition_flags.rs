#[derive(Debug, Clone, Copy)]
pub enum ConditionFlag {
    Pos = 1 << 0, /* P */
    Zro = 1 << 1, /* Z */
    Neg = 1 << 2, /* N */
}

impl ConditionFlag {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(ConditionFlag::Pos),
            2 => Some(ConditionFlag::Zro),
            4 => Some(ConditionFlag::Neg),
            _ => None,
        }
    }

    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}
