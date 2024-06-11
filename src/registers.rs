// Module for the registers of the LC3

#[derive(Debug, Clone, Copy)]
enum Register {
    R0 = 0, /* general purpose registers */
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    Cond, /* condition flag */
    Count /* count register */
}

impl Register {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(Register::R0),
            1 => Some(Register::R1),
            2 => Some(Register::R2),
            3 => Some(Register::R3),
            4 => Some(Register::R4),
            5 => Some(Register::R5),
            6 => Some(Register::R6),
            7 => Some(Register::R7),
            8 => Some(Register::Cond),
            9 => Some(Register::Count),
            _ => None,
        }
    }
}
