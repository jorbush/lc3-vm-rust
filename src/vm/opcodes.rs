// Module for the opcodes of the LC3

#[derive(Debug, Clone, Copy)]
pub(crate) enum OpCode {
    Br = 0, /* branch */
    Add,    /* add  */
    Ld,     /* load */
    St,     /* store */
    Jsr,    /* jump register */
    And,    /* bitwise and */
    Ldr,    /* load register */
    Str,    /* store register */
    Rti,    /* unused */
    Not,    /* bitwise not */
    Ldi,    /* load indirect */
    Sti,    /* store indirect */
    Jmp,    /* jump */
    Res,    /* reserved (unused) */
    Lea,    /* load effective address */
    Trap,   /* execute trap */
}

impl OpCode {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(OpCode::Br),
            1 => Some(OpCode::Add),
            2 => Some(OpCode::Ld),
            3 => Some(OpCode::St),
            4 => Some(OpCode::Jsr),
            5 => Some(OpCode::And),
            6 => Some(OpCode::Ldr),
            7 => Some(OpCode::Str),
            8 => Some(OpCode::Rti),
            9 => Some(OpCode::Not),
            10 => Some(OpCode::Ldi),
            11 => Some(OpCode::Sti),
            12 => Some(OpCode::Jmp),
            13 => Some(OpCode::Res),
            14 => Some(OpCode::Lea),
            15 => Some(OpCode::Trap),
            _ => None,
        }
    }

    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}
