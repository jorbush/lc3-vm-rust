pub(crate) enum TrapCode {
    Getc = 0x20,  /* get character from keyboard, not echoed onto the terminal */
    Out = 0x21,   /* output a character */
    Puts = 0x22,  /* output a word string */
    In = 0x23,    /* get character from keyboard, echoed onto the terminal */
    Putsp = 0x24, /* output a byte string */
    Halt = 0x25,  /* halt the program */
}

impl From<TrapCode> for u16 {
    fn from(val: TrapCode) -> Self {
        val as u16
    }
}

impl TryFrom<u16> for TrapCode {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0x20 => Self::Getc,
            0x21 => Self::Out,
            0x22 => Self::Puts,
            0x23 => Self::In,
            0x24 => Self::Putsp,
            0x25 => Self::Halt,
            _ => return Err("invalid trap code"),
        })
    }
}
