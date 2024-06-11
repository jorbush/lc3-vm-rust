// Module for the opcodes of the LC3

#[derive(Debug, Clone, Copy)]
enum OpCode {
    BR = 0,  /* branch */
    ADD,     /* add  */
    LD,      /* load */
    ST,      /* store */
    JSR,     /* jump register */
    AND,     /* bitwise and */
    LDR,     /* load register */
    STR,     /* store register */
    RTI,     /* unused */
    NOT,     /* bitwise not */
    LDI,     /* load indirect */
    STI,     /* store indirect */
    JMP,     /* jump */
    RES,     /* reserved (unused) */
    LEA,     /* load effective address */
    TRAP     /* execute trap */
}

impl OpCode {
    fn from_u16(value: u16) -> Option<Self> {
        match value {
            0 => Some(OpCode::BR),
            1 => Some(OpCode::ADD),
            2 => Some(OpCode::LD),
            3 => Some(OpCode::ST),
            4 => Some(OpCode::JSR),
            5 => Some(OpCode::AND),
            6 => Some(OpCode::LDR),
            7 => Some(OpCode::STR),
            8 => Some(OpCode::RTI),
            9 => Some(OpCode::NOT),
            10 => Some(OpCode::LDI),
            11 => Some(OpCode::STI),
            12 => Some(OpCode::JMP),
            13 => Some(OpCode::RES),
            14 => Some(OpCode::LEA),
            15 => Some(OpCode::TRAP),
            _ => None,
        }
    }
}
