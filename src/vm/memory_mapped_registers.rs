// Module for the memory mapped registers of the LC3

#[derive(Debug, Clone, Copy)]
pub enum MemoryMappedRegister {
    Kbsr = 0xFE00,  /* keyboard status */
    Kbddr = 0xFE02, /* keyboard data */
}

impl From<MemoryMappedRegister> for u16 {
    fn from(val: MemoryMappedRegister) -> Self {
        val as u16
    }
}

impl TryFrom<u16> for MemoryMappedRegister {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match value {
            0xFE00 => Self::Kbsr,
            0xFE02 => Self::Kbddr,
            _ => return Err("invalid memory mapped register"),
        })
    }
}

impl From<MemoryMappedRegister> for usize {
    fn from(val: MemoryMappedRegister) -> Self {
        val as usize
    }
}
