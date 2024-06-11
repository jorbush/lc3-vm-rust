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
        vm.registers[Register::Cond.to_usize()] = ConditionFlag::Zro.to_u16();
        /* set the PC to starting position */
        vm.registers[Register::PC.to_usize()] = PC_START;
        vm
    }

    fn fetch(&self) -> u16 {
        self.memory[self.registers[Register::PC.to_usize()] as usize]
    }

    fn decode_and_execute(&mut self, instr: u16) {
        let op = OpCode::from_u16(instr >> 12).unwrap_or(OpCode::Res);

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
            self.registers[Register::PC.to_usize()] += 1;
            self.decode_and_execute(instr);
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

    fn add(&mut self, instr: u16) { /* TODO */
    }
    fn and(&mut self, instr: u16) { /* TODO */
    }
    fn not(&mut self, instr: u16) { /* TODO */
    }
    fn br(&mut self, instr: u16) { /* TODO */
    }
    fn jmp(&mut self, instr: u16) { /* TODO */
    }
    fn jsr(&mut self, instr: u16) { /* TODO */
    }
    fn ld(&mut self, instr: u16) { /* TODO */
    }
    fn ldi(&mut self, instr: u16) { /* TODO */
    }
    fn ldr(&mut self, instr: u16) { /* TODO */
    }
    fn lea(&mut self, instr: u16) { /* TODO */
    }
    fn st(&mut self, instr: u16) { /* TODO */
    }
    fn sti(&mut self, instr: u16) { /* TODO */
    }
    fn str(&mut self, instr: u16) { /* TODO */
    }
    fn trap(&mut self, instr: u16) { /* TODO */
    }
}
