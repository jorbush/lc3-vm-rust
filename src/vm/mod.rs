mod condition_flags;
mod opcodes;
mod registers;

use condition_flags::*;
use opcodes::OpCode;
use registers::*;

const MEMORY_SIZE: usize = 65536; /* 65536 locations */
const PC_START: u16 = 0x3000;

pub(crate) struct VM {
    memory: [u16; MEMORY_SIZE],
    registers: [u16; 10],
    running: bool,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            memory: [0; 65536],
            registers: [0; 10],
            running: true,
        };
        vm.registers[Register::Cond.to_usize()] = ConditionFlag::Zro.to_u16();
        vm.registers[Register::PC.to_usize()] = PC_START;
        vm
    }
}
