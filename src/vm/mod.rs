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
            OpCode::Rti | OpCode::Res => self.abort(),
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

    fn update_flags(&mut self, reg_index: usize) {
        if self.registers[reg_index] == 0 {
            self.registers[usize::from(Register::Cond)] = ConditionFlag::Zro.into();
        } else if self.registers[reg_index] >> 15 == 1 {
            /* a 1 in the left-most bit indicates negative */
            self.registers[usize::from(Register::Cond)] = ConditionFlag::Neg.into();
        } else {
            self.registers[usize::from(Register::Cond)] = ConditionFlag::Pos.into();
        }
    }

    fn add(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 | 5 | 4 3 2 1 0
                0 0 0 1 |   DR    |  SR1  | 0 | 0 0 | SR2
                0 0 0 1 |   DR    |  SR1  | 1 |   imm5
        */
        /* extract destination register (DR) */
        let dr = (instr >> 9) & 0x7;
        /* extract first operand (SR1) */
        let sr1 = (instr >> 6) & 0x7;
        /* whether we are in immediate mode */
        let imm_flag = (instr >> 5) & 0x1;

        if imm_flag != 0 {
            // immediate mode
            let imm5 = Self::sign_extend(instr & 0x1F, 5);
            self.registers[dr as usize] = self.registers[sr1 as usize].wrapping_add(imm5);
        } else {
            // register mode
            let sr2 = instr & 0x7;
            self.registers[dr as usize] =
                self.registers[sr1 as usize].wrapping_add(self.registers[sr2 as usize]);
        }

        self.update_flags(dr as usize);
    }

    fn and(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 | 5 | 4 3 2 1 0
                0 1 0 1 |   DR    |  SR1  | 0 | 0 0 | SR2
                0 1 0 1 |   DR    |  SR1  | 1 |   imm5
        */
        let dr = (instr >> 9) & 0x7;
        let sr1 = (instr >> 6) & 0x7;
        let imm_flag = (instr >> 5) & 0x1;

        if imm_flag != 0 {
            let imm5 = Self::sign_extend(instr & 0x1F, 5);
            self.registers[dr as usize] = self.registers[sr1 as usize] & imm5;
        } else {
            let sr2 = instr & 0x7;
            self.registers[dr as usize] =
                self.registers[sr1 as usize] & self.registers[sr2 as usize];
        }
    }

    fn ldi(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0
                1 0 1 0 |   DR    |  PCoffset9
        */
        let dr = (instr >> 9) & 0x7;
        /* PCoffset 9*/
        let pc_offset = Self::sign_extend(instr & 0x1FF, 9);
        /* add pc_offset to the current PC, look at that memory location to get the final address */
        let address = self.registers[usize::from(Register::PC)].wrapping_add(pc_offset);
        let effective_address = self.memory[address as usize];
        self.registers[dr as usize] = self.memory[effective_address as usize];
        self.update_flags(dr as usize);
    }

    fn not(&mut self, instr: u16) {
        /*
           15 14 13 12 | 11 10 9 | 8 7 6 | 5 | 4 3 2 1 0
               1 0 0 1 |   DR    |  SR   | 1 | 1 1 1 1 1
        */
        let dr = (instr >> 9) & 0x7;
        let sr = (instr >> 6) & 0x7;
        self.registers[dr as usize] = !self.registers[sr as usize];
        self.update_flags(dr as usize);
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

    fn abort(&mut self) {
        println!("Bad Opcode!");
        println!("Aborting the VM...");
        self.running = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_register_mode() {
        let mut vm = VM::new();
        // Set initial values for the registers
        vm.registers[1] = 5; // SR1
        vm.registers[2] = 10; // SR2
        println!("Registers before ADD: {:?}", vm.registers);

        // Create an ADD instruction: DR = 0, SR1 = 1, SR2 = 2
        // Binary representation: 0001 000 001 000 010
        let instr: u16 = 0b0001_0000_0100_0010;

        vm.add(instr);

        println!("Registers after ADD: {:?}", vm.registers);
        assert_eq!(vm.registers[0], 15);
    }

    #[test]
    fn test_add_immediate_mode() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[1] = 5; // SR1
        println!("Registers before ADD: {:?}", vm.registers);

        // Create an ADD instruction: DR = 0, SR1 = 1, imm5 = 10
        // Binary representation: 0001 000 001 1 01010
        let instr: u16 = 0b0001_0000_0110_1010;

        vm.add(instr);

        println!("Registers after ADD: {:?}", vm.registers);
        assert_eq!(vm.registers[0], 15);
    }

    #[test]
    fn test_ldi() {
        let mut vm = VM::new();
        // Set initial value for the memory
        vm.memory[0x3002] = 0x3050; // Memory at PC + offset (for LDI)
        vm.memory[0x3050] = 20; // Memory at address 0x3050 (final address)
        println!("Registers before LDI: {:?}", vm.registers);

        // Set the PC to 0x3000
        vm.registers[usize::from(Register::PC)] = 0x3000;

        // Create an LDI instruction: DR = 0, PCoffset9 = 2
        // Binary representation: 1010 000 000 000010
        let instr: u16 = 0b1010_0000_0000_0010;

        vm.ldi(instr);

        println!("Registers after LDI: {:?}", vm.registers);
        println!("Memory after LDI: {:?}", &vm.memory[0x3000..0x3060]);
        assert_eq!(vm.registers[0], 20);
    }

    #[test]
    fn test_rti() {
        let mut vm = VM::new();
        println!("Registers before RTI: {:?}", vm.registers);

        // Create an RTI instruction
        // Binary representation: 1000 0000 0000 0000
        let instr: u16 = 0b1000_0000_0000_0000;

        vm.execute(OpCode::Rti, instr);

        println!("Registers after RTI: {:?}", vm.registers);
        assert!(!vm.running);
    }

    #[test]
    fn test_res() {
        let mut vm = VM::new();
        println!("Registers before RES: {:?}", vm.registers);

        // Create a RES instruction
        // Binary representation: 1110 0000 0000 0000
        let instr: u16 = 0b1101_0000_0000_0000;

        vm.execute(OpCode::Res, instr);

        println!("Registers after RES: {:?}", vm.registers);
        assert!(!vm.running);
    }

    #[test]
    fn test_and_register_mode() {
        let mut vm = VM::new();
        // Set initial values for the registers

        vm.registers[1] = 0b1010; // SR1
        vm.registers[2] = 0b1100; // SR2
        println!("Registers before AND: {:?}", vm.registers);

        // Create an AND instruction: DR = 0, SR1 = 1, SR2 = 2
        // Binary representation: 0101 000 001 000 010
        let instr: u16 = 0b0101_0000_0100_0010;

        vm.and(instr);

        println!("Registers after AND: {:?}", vm.registers);
        assert_eq!(vm.registers[0], 0b1000);
    }

    #[test]
    fn test_and_immediate_mode() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[1] = 0b1010; // SR1
        println!("Registers before AND: {:?}", vm.registers);

        // Create an AND instruction: DR = 0, SR1 = 1, imm5 = 0b1100
        // Binary representation: 0101 000 001 1 01100
        let instr: u16 = 0b0101_0000_0110_1100;

        vm.and(instr);

        println!("Registers after AND: {:?}", vm.registers);
        assert_eq!(vm.registers[0], 0b1000);
    }
}
