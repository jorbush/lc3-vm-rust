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
        /*
           15 14 13 12 | 11  10  9 | 8 7 6 5 4 3 2 1 0
               0 0 0 0 | n | z | p |  PCoffset9
        */
        let n = (instr >> 11) & 0x1;
        let z = (instr >> 10) & 0x1;
        let p = (instr >> 9) & 0x1;
        let cond = self.registers[usize::from(Register::Cond)];
        if (n != 0 && cond == ConditionFlag::Neg.into())
            || (z != 0 && cond == ConditionFlag::Zro.into())
            || (p != 0 && cond == ConditionFlag::Pos.into())
        {
            let pc_offset = Self::sign_extend(instr & 0x1FF, 9);
            self.registers[usize::from(Register::PC)] =
                self.registers[usize::from(Register::PC)].wrapping_add(pc_offset);
        }
    }

    fn jmp(&mut self, instr: u16) {
        /*
                15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0
            JMP     1 1 0 0 | 0  0  0 | BaseR | 0 0 0 0 0 0
            RET     1 1 0 0 | 0  0  0 | 1 1 1 | 0 0 0 0 0 0
        */
        let base_r = (instr >> 6) & 0x7;
        self.registers[usize::from(Register::PC)] = self.registers[base_r as usize];
    }

    fn jsr(&mut self, instr: u16) {
        /*
                15 14 13 12 | 11 | 10 9 8 7 6 | 5 4 3 2 1 0
            JSR     0 1 0 0 |  1 |      PCoffset11
            JSRR    0 1 0 0 |  0 | 0 0 | BaseR | 0 0 0 0 0 0
        */
        // First, the incremented PC is saved in R7.
        // This is the linkage back to the calling routine.
        self.registers[usize::from(Register::R7)] = self.registers[usize::from(Register::PC)];
        let long_flag = (instr >> 11) & 0x1;
        if long_flag != 0 {
            // JSR
            let long_pc_offset = Self::sign_extend(instr & 0x7FF, 11);
            self.registers[usize::from(Register::PC)] =
                self.registers[usize::from(Register::PC)].wrapping_add(long_pc_offset);
        } else {
            // JSRR
            let base_r = (instr >> 6) & 0x7;
            self.registers[usize::from(Register::PC)] = self.registers[base_r as usize];
        }
    }

    fn ld(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0
                0 0 1 0 |   DR    |  PCoffset9
        */
        let dr = (instr >> 9) & 0x7;
        let pc_offset = Self::sign_extend(instr & 0x1FF, 9);
        let address = self.registers[usize::from(Register::PC)].wrapping_add(pc_offset);
        self.registers[dr as usize] = self.memory[address as usize];
        self.update_flags(dr as usize);
    }

    fn ldr(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0
                0 1 1 0 |    DR   | BaseR | 6-bit offset
        */
        let dr = (instr >> 9) & 0x7;
        let base_r = (instr >> 6) & 0x7;
        let offset = Self::sign_extend(instr & 0x3F, 6);
        let address = self.registers[base_r as usize].wrapping_add(offset);
        self.registers[dr as usize] = self.memory[address as usize];
        self.update_flags(dr as usize);
    }

    fn lea(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0
                1 1 1 0 |    DR   |  PCoffset9
        */
        let dr = (instr >> 9) & 0x7;
        let pc_offset = Self::sign_extend(instr & 0x1FF, 9);
        let address = self.registers[usize::from(Register::PC)].wrapping_add(pc_offset);
        self.registers[dr as usize] = address;
        self.update_flags(dr as usize);
    }

    fn st(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0
                0 0 1 1 |    SR   |  PCoffset9
        */
        let sr = (instr >> 9) & 0x7;
        let pc_offset = Self::sign_extend(instr & 0x1FF, 9);
        let address = self.registers[usize::from(Register::PC)].wrapping_add(pc_offset);
        self.memory[address as usize] = self.registers[sr as usize];
    }

    fn sti(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 5 4 3 2 1 0
                1 0 1 1 |    SR   |  PCoffset9
        */
        let sr = (instr >> 9) & 0x7;
        let pc_offset = Self::sign_extend(instr & 0x1FF, 9);
        let address = self.registers[usize::from(Register::PC)].wrapping_add(pc_offset);
        let effective_address = self.memory[address as usize];
        self.memory[effective_address as usize] = self.registers[sr as usize];
    }

    fn str(&mut self, instr: u16) {
        /*
            15 14 13 12 | 11 10 9 | 8 7 6 | 5 4 3 2 1 0
                0 1 1 1 |    SR   | BaseR | offset6
        */
        let sr = (instr >> 9) & 0x7;
        let base_r = (instr >> 6) & 0x7;
        let offset = Self::sign_extend(instr & 0x3F, 6);
        let address = self.registers[base_r as usize].wrapping_add(offset);
        self.memory[address as usize] = self.registers[sr as usize];
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

    #[test]
    fn test_not() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[1] = 0b1010; // SR
        println!("Registers before NOT: {:?}", vm.registers);

        // Create a NOT instruction: DR = 0, SR = 1
        // Binary representation: 1001 000 001 111111
        let instr: u16 = 0b1001_0000_0111_1111;

        vm.not(instr);

        println!("Registers after NOT: {:?}", vm.registers);
        assert_eq!(vm.registers[0], !0b1010);
    }

    #[test]
    fn test_br() {
        let mut vm = VM::new();
        // Set initial value for the condition flag
        vm.registers[usize::from(Register::Cond)] = ConditionFlag::Neg.into();
        // Set initial value for the PC
        vm.registers[usize::from(Register::PC)] = 0x3000;
        // Set initial value for the memory
        println!("Registers before BR: {:?}", vm.registers);

        // Create a BR instruction: n = 1, z = 0, p = 0, PCoffset9 = 2
        // Binary representation: 0000 100 000 000010
        let instr: u16 = 0b0000_1000_0000_0010;

        vm.br(instr);

        println!("Registers after BR: {:?}", vm.registers);
        println!("Memory after BR: {:?}", &vm.memory[0x3000..0x3002]);
        assert_eq!(vm.registers[usize::from(Register::PC)], 0x3002);
    }

    #[test]
    fn test_jmp() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[1] = 0x3002; // BaseR
        println!("Registers before JMP: {:?}", vm.registers);

        // Create a JMP instruction: BaseR = 1
        // Binary representation: 1100 000 001 000000
        let instr: u16 = 0b1100_0000_0100_0000;

        vm.jmp(instr);

        println!("Registers after JMP: {:?}", vm.registers);
        assert_eq!(vm.registers[usize::from(Register::PC)], 0x3002);
    }

    #[test]
    fn test_ret() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[7] = 0x4000; // R7 is the link register
        println!("Registers before RET: {:?}", vm.registers);

        // Create a RET instruction
        // Binary representation: 1100 000 111 000000
        let instr: u16 = 0b1100_0001_1100_0000;

        vm.jmp(instr);

        println!("Registers after RET: {:?}", vm.registers);
        assert_eq!(vm.registers[usize::from(Register::PC)], 0x4000);
    }

    #[test]
    fn test_jsr() {
        let mut vm = VM::new();
        // Set initial value for the PC
        vm.registers[usize::from(Register::PC)] = PC_START;
        println!("Registers before JSR: {:?}", vm.registers);

        // Create a JSR instruction: PCoffset11 = 2
        // Binary representation: 0100 1 000000000010
        let instr: u16 = 0b0100_1000_0000_0010;

        vm.jsr(instr);

        println!("Registers after JSR: {:?}", vm.registers);
        assert_eq!(vm.registers[usize::from(Register::R7)], 0x3000);
        assert_eq!(vm.registers[usize::from(Register::PC)], 0x3002);
    }

    #[test]
    fn test_jsrr() {
        let mut vm = VM::new();
        // Set initial value for the registers
        vm.registers[usize::from(Register::PC)] = PC_START;
        vm.registers[1] = 0x3002; // BaseR
        println!("Registers before JSRR: {:?}", vm.registers);

        // Create a JSRR instruction: BaseR = 1
        // Binary representation: 0100 0 00 001 000000
        let instr: u16 = 0b0100_0000_0100_0000;

        vm.jsr(instr);

        println!("Registers after JSRR: {:?}", vm.registers);
        assert_eq!(vm.registers[usize::from(Register::R7)], 0x3000);
        assert_eq!(vm.registers[usize::from(Register::PC)], 0x3002);
    }

    #[test]
    fn test_ld() {
        let mut vm = VM::new();
        // Set initial value for the memory
        vm.memory[0x3002] = 20; // Memory at PC + offset (for LD)
        vm.registers[usize::from(Register::PC)] = PC_START;
        println!("Registers before LD: {:?}", vm.registers);

        // Create an LD instruction: DR = 0, PCoffset9 = 2
        // Binary representation: 0010 000 000 000010
        let instr: u16 = 0b0010_0000_0000_0010;

        vm.ld(instr);

        println!("Registers after LD: {:?}", vm.registers);
        println!("Memory after LD: {:?}", &vm.memory[0x3000..0x3002]);
        assert_eq!(vm.registers[0], 20);
    }

    #[test]
    fn test_ldr() {
        let mut vm = VM::new();
        // Set initial value for the memory
        vm.memory[0x3002] = 20; // Memory at BaseR + offset (for LDR)
        vm.registers[1] = 0x3000; // BaseR
        println!("Registers before LDR: {:?}", vm.registers);

        // Create an LDR instruction: DR = 0, BaseR = 1, offset = 2
        // Binary representation: 0110 000 001 000010
        let instr: u16 = 0b0110_0000_0100_0010;

        vm.ldr(instr);

        println!("Registers after LDR: {:?}", vm.registers);
        println!("Memory after LDR: {:?}", &vm.memory[0x3000..0x3002]);
        assert_eq!(vm.registers[0], 20);
    }

    #[test]
    fn test_lea() {
        let mut vm = VM::new();
        // Set initial value for the PC
        vm.registers[usize::from(Register::PC)] = PC_START;
        println!("Registers before LEA: {:?}", vm.registers);

        // Create a LEA instruction: DR = 0, PCoffset9 = 2
        // Binary representation: 1110 000 000 000010
        let instr: u16 = 0b1110_0000_0000_0010;

        vm.lea(instr);

        println!("Registers after LEA: {:?}", vm.registers);
        assert_eq!(vm.registers[0], 0x3002);
    }

    #[test]
    fn test_st() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[0] = 20; // SR
        vm.registers[usize::from(Register::PC)] = PC_START;
        println!("Registers before ST: {:?}", vm.registers);

        // Create a ST instruction: SR = 0, PCoffset9 = 2
        // Binary representation: 0011 000 000 000010
        let instr: u16 = 0b0011_0000_0000_0010;

        vm.st(instr);

        println!("Registers after ST: {:?}", vm.registers);
        println!("Memory after ST: {:?}", &vm.memory[0x3000..0x3002]);
        assert_eq!(vm.memory[0x3002], 20);
    }

    #[test]
    fn test_sti() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[0] = 20; // SR
        vm.memory[0x3002] = 0x3050; // Memory at PC + offset (for STI)
        vm.registers[usize::from(Register::PC)] = PC_START;
        println!("Registers before STI: {:?}", vm.registers);

        // Create a STI instruction: SR = 0, PCoffset9 = 2
        // Binary representation: 1011 000 000 000010
        let instr: u16 = 0b1011_0000_0000_0010;

        vm.sti(instr);

        println!("Registers after STI: {:?}", vm.registers);
        println!("Memory after STI: {:?}", &vm.memory[0x3000..0x3060]);
        assert_eq!(vm.memory[0x3050], 20);
    }

    #[test]
    fn test_str() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[0] = 20; // SR
        vm.registers[1] = 0x3000; // BaseR
        println!("Registers before STR: {:?}", vm.registers);

        // Create a STR instruction: SR = 0, BaseR = 1, offset = 2
        // Binary representation: 0111 000 001 000010
        let instr: u16 = 0b0111_0000_0100_0010;

        vm.str(instr);

        println!("Registers after STR: {:?}", vm.registers);
        println!("Memory after STR: {:?}", &vm.memory[0x3000..0x3002]);
        assert_eq!(vm.memory[0x3002], 20);
    }
}
