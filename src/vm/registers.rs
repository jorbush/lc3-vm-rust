// Module for the registers of the LC3

#[derive(Debug, Clone, Copy)]
pub enum Register {
    R0 = 0, /* general purpose registers */
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,    /* program counter */
    Cond,  /* condition flag */
    Count, /* count register */
}

impl From<Register> for usize {
    fn from(val: Register) -> Self {
        match val {
            Register::R0 => 0,
            Register::R1 => 1,
            Register::R2 => 2,
            Register::R3 => 3,
            Register::R4 => 4,
            Register::R5 => 5,
            Register::R6 => 6,
            Register::R7 => 7,
            Register::PC => 8,
            Register::Cond => 9,
            Register::Count => 10,
        }
    }
}

impl TryFrom<usize> for Register {
    type Error = &'static str;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::R0,
            1 => Self::R1,
            2 => Self::R2,
            3 => Self::R3,
            4 => Self::R4,
            5 => Self::R5,
            6 => Self::R6,
            7 => Self::R7,
            8 => Self::PC,
            9 => Self::Cond,
            10 => Self::Count,
            _ => return Err("invalid register"),
        })
    }
}

impl From<Register> for u16 {
    fn from(val: Register) -> Self {
        val as u16
    }
}
