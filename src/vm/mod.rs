mod condition_flags;
mod opcodes;
mod registers;

use condition_flags::*;
use opcodes::OpCode;
use registers::*;

use std::fs::File;
use std::io::{self, Read};

const MEMORY_SIZE: usize = 65536; /* 65536 locations */

/* 0x3000 is the default */
const PC_START: u16 = 0x3000;

pub(crate) struct VM {
    memory: [u16; MEMORY_SIZE],
    registers: [u16; 10],
    running: bool,
}

impl VM {
    pub fn new() -> Self {
        let mut vm = VM {
            memory: [0; MEMORY_SIZE],
            registers: [0; 10],
            running: true,
        };
        /* since exactly one condition flag should be set at any given time, set the Z flag */
        vm.registers[usize::from(Register::Cond)] = ConditionFlag::Zro.into();
        /* set the PC to starting position */
        vm.registers[usize::from(Register::PC)] = PC_START;
        vm
    }

    fn fetch(&self) -> u16 {
        self.memory[self.registers[usize::from(Register::PC)] as usize]
    }

    fn decode(instr: u16) -> OpCode {
        OpCode::try_from(instr >> 12).unwrap()
    }

    fn execute(&mut self, op: OpCode, instr: u16) {
        match op {
            OpCode::Add => self.add(instr),
            OpCode::And => self.and(instr),
            OpCode::Not => self.not(instr),
            OpCode::Br => self.br(instr),
            OpCode::Jmp => self.jmp(instr),
            OpCode::Jsr => self.jsr(instr),
            OpCode::Ld => self.ld(instr),
            OpCode::Ldi => self.ldi(instr),
            OpCode::Ldr => self.ldr(instr),
            OpCode::Lea => self.lea(instr),
            OpCode::St => self.st(instr),
            OpCode::Sti => self.sti(instr),
            OpCode::Str => self.str(instr),
            OpCode::Trap => self.trap(instr),
            _ => self.running = false,
        }
    }

    pub fn run(&mut self) {
        while self.running {
            let instr = self.fetch();
            self.registers[usize::from(Register::PC)] += 1;
            let op = Self::decode(instr);
            self.execute(op, instr);
        }
    }

    pub fn load_image(&mut self, path: &str) -> io::Result<()> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let origin = u16::from_be_bytes([buffer[0], buffer[1]]) as usize;
        let mut memory_index = origin;

        for chunk in buffer[2..].chunks(2) {
            let value = u16::from_be_bytes([chunk[0], chunk[1]]);
            self.memory[memory_index] = value;
            memory_index += 1;
        }

        Ok(())
    }

    fn sign_extend(x: u16, bit_count: u16) -> u16 {
        // if the leftmost bit is 1, then it's negative
        if (x >> (bit_count - 1)) & 1 == 1 {
            // set the leftmost bits to 1
            x | (0xFFFF << bit_count)
        } else {
            x
        }
    }

    fn add(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction ADD ({:#x}) not implemented yet.", instr)
        );
    }

    fn and(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction AND ({:#x}) not implemented yet.", instr)
        );
    }

    fn not(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction NOT ({:#x}) not implemented yet.", instr)
        );
    }

    fn br(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction BR ({:#x}) not implemented yet.", instr)
        );
    }

    fn jmp(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction JMP ({:#x}) not implemented yet.", instr)
        );
    }

    fn jsr(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction JSR ({:#x}) not implemented yet.", instr)
        );
    }

    fn ld(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction LD ({:#x}) not implemented yet.", instr)
        );
    }

    fn ldi(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction LDI ({:#x}) not implemented yet.", instr)
        );
    }

    fn ldr(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction LDR ({:#x}) not implemented yet.", instr)
        );
    }

    fn lea(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction LEA ({:#x}) not implemented yet.", instr)
        );
    }

    fn st(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction ST ({:#x}) not implemented yet.", instr)
        );
    }

    fn sti(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction STI ({:#x}) not implemented yet.", instr)
        );
    }

    fn str(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction STR ({:#x}) not implemented yet.", instr)
        );
    }

    fn trap(&mut self, instr: u16) {
        todo!(
            "{}",
            format!("Instruction TRAP ({:#x}) not implemented yet.", instr)
        );
    }
}
