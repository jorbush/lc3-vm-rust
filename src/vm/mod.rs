mod condition_flags;
mod memory_mapped_registers;
mod opcodes;
mod registers;
mod trap_codes;

use crate::utils::terminal;
use condition_flags::*;
use libc::c_int;
use memory_mapped_registers::MemoryMappedRegister;
use opcodes::OpCode;
use registers::*;
use std::io::{self, Read, Write};
use trap_codes::TrapCode;

extern "C" {
    fn getchar() -> c_int;
}

pub fn get_char() -> i32 {
    unsafe { getchar() }
}

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

    fn decode(instr: u16) -> OpCode {
        OpCode::try_from(instr >> 12).unwrap()
    }

    fn fetch(&mut self) -> u16 {
        self.mem_read(self.registers[usize::from(Register::PC)])
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
            let instr: u16 = self.fetch();
            self.registers[usize::from(Register::PC)] += 1;
            let op = Self::decode(instr);
            self.execute(op, instr);
        }
    }

    pub fn load_image(&mut self, path: &str) -> io::Result<()> {
        self.read_image(path)
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
        let effective_address = self.mem_read(address);
        self.registers[dr as usize] = self.mem_read(effective_address);
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
        /*
            15 14 13 12 | 11 10 9 8 7 6 5 4 3 2 1 0
                1 1 1 1 | 0 0 0 0 |   trapvect8
        */
        terminal::turn_off_canonical_and_echo_modes();
        let trap_vect = instr & 0xFF;
        match trap_vect.try_into().unwrap() {
            TrapCode::Getc => self.trap_getc(),
            TrapCode::Out => self.trap_out(),
            TrapCode::Puts => self.trap_puts(),
            TrapCode::In => self.trap_in(),
            TrapCode::Putsp => self.trap_puts_p(),
            TrapCode::Halt => self.trap_halt(),
        }
        terminal::restore_terminal_settings();
    }

    fn abort(&mut self) {
        println!("Bad Opcode!");
        println!("Aborting the VM...");
        self.running = false;
    }

    fn trap_getc(&mut self) {
        let register_index = usize::from(Register::R0);
        self.registers[register_index] = get_char() as u16;
        self.update_flags(register_index);
    }

    fn trap_out(&mut self) {
        print!(
            "{}",
            self.registers[usize::from(Register::R0)] as u8 as char
        );
        io::stdout().flush().expect("Flushed.");
    }

    fn trap_puts(&mut self) {
        let mut address = self.registers[usize::from(Register::R0)];
        while self.memory[address as usize] != 0x0000 {
            print!("{}", self.memory[address as usize] as u8 as char);
            address += 1;
        }
        io::stdout().flush().expect("Flushed.");
    }

    fn trap_in(&mut self) {
        print!("Enter a character: ");
        io::stdout().flush().expect("Flushed.");
        let register_index = usize::from(Register::R0);
        self.registers[register_index] = get_char() as u16;
        self.update_flags(register_index);
    }

    fn trap_puts_p(&mut self) {
        /* one char per byte (two bytes per word)
        here we need to swap back to
        big endian format */
        let mut address = self.registers[usize::from(Register::R0)];
        while self.memory[address as usize] != 0x0000 {
            let c = self.memory[address as usize];
            let c1 = (c & 0xFF) as u8 as char;
            print!("{}", c1);
            let c2 = (c >> 8) as u8 as char;
            if c2 != '\0' {
                print!("{}", c2);
            }
            address += 1;
        }
        io::stdout().flush().expect("Flushed.");
    }

    fn trap_halt(&mut self) {
        println!("Halting the VM...");
        self.running = false;
        io::stdout().flush().expect("Flushed.");
    }

    fn read_image_file(&mut self, file: &mut std::fs::File) -> std::io::Result<()> {
        // Read the origin address
        let mut origin_buf = [0; 2];
        file.read_exact(&mut origin_buf)?;
        let origin = u16::from_be_bytes(origin_buf) as usize;

        // Read the file content into memory
        let max_read = MEMORY_SIZE - origin;
        let mut buffer = vec![0; max_read * 2];
        let bytes_read = file.read(&mut buffer)?;

        // Convert and copy the data into memory
        for i in 0..(bytes_read / 2) {
            // let word = Self::swap16(u16::from_be_bytes([buffer[2 * i], buffer[2 * i + 1]]));
            let word = u16::from_be_bytes([buffer[2 * i], buffer[2 * i + 1]]);
            self.mem_write(origin + i, word);
        }
        Ok(())
    }

    fn read_image(&mut self, image_path: &str) -> std::io::Result<()> {
        let mut file = std::fs::File::open(image_path)?;
        self.read_image_file(&mut file)
    }

    fn mem_write(&mut self, address: usize, value: u16) {
        self.memory[address] = value;
    }

    fn mem_read(&mut self, address: u16) -> u16 {
        if address == MemoryMappedRegister::Kbsr.into() {
            let mut buffer = [0; 1];
            std::io::stdin().read_exact(&mut buffer).unwrap();
            if buffer[0] != 0 {
                self.memory[usize::from(MemoryMappedRegister::Kbsr)] = 1 << 15;
                self.memory[usize::from(MemoryMappedRegister::Kbddr)] = get_char() as u16;
            } else {
                self.memory[usize::from(MemoryMappedRegister::Kbsr)] = 0;
            }
        }
        self.memory[address as usize]
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use io::Write;

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

    #[test]
    fn test_trap_puts() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[0] = 0x3000; // R0
        vm.memory[0x3000] = 'H' as u16;
        vm.memory[0x3001] = 'e' as u16;
        vm.memory[0x3002] = 'l' as u16;
        vm.memory[0x3003] = 'l' as u16;
        vm.memory[0x3004] = 'o' as u16;
        vm.memory[0x3005] = ' ' as u16;
        vm.memory[0x3006] = 'W' as u16;
        vm.memory[0x3007] = 'o' as u16;
        vm.memory[0x3008] = 'r' as u16;
        vm.memory[0x3009] = 'l' as u16;
        vm.memory[0x300A] = 'd' as u16;
        vm.memory[0x300B] = '!' as u16;
        vm.memory[0x300C] = 0x0000; // Null-terminated string
        println!("Registers before TRAP: {:?}", vm.registers);

        vm.trap_puts();

        println!("Registers after TRAP: {:?}", vm.registers);
        println!("Memory after TRAP: {:?}", &vm.memory[0x3000..0x300D]);
        assert_eq!(
            &vm.memory[0x3000..0x300D],
            &[
                0x0048, 0x0065, 0x006C, 0x006C, 0x006F, 0x0020, 0x0057, 0x006F, 0x0072, 0x006C,
                0x0064, 0x0021, 0x0000
            ]
        );
    }

    // #[test]
    // fn test_trap_getc() {
    //     let mut vm = VM::new();
    //     // Set initial value for the register
    //     vm.registers[0] = 0x0000; // R0
    //     println!("Registers before TRAP: {:?}", vm.registers);

    //     vm.trap_in();

    //     println!("Registers after TRAP: {:?}", vm.registers);
    //     assert_eq!(vm.registers[0], 'a' as u16);
    // }

    #[test]
    fn test_trap_out() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[0] = 'a' as u16; // R0
        println!("Registers before TRAP: {:?}", vm.registers);

        vm.trap_out();

        println!("Registers after TRAP: {:?}", vm.registers);
        assert_eq!(vm.registers[0], 'a' as u16);
    }

    // #[test]
    // fn test_trap_in() {
    //     let mut vm = VM::new();
    //     // Set initial value for the register
    //     vm.registers[0] = 0x0000; // R0
    //     println!("Registers before TRAP: {:?}", vm.registers);

    //     vm.trap_in();

    //     println!("Registers after TRAP: {:?}", vm.registers);
    //     assert_eq!(vm.registers[0], 'a' as u16);
    // }

    #[test]
    fn test_trap_puts_p() {
        let mut vm = VM::new();
        // Set initial value for the register
        vm.registers[0] = 0x3000; // R0
        vm.memory[0x3000] = 0x4848; // "HH"
        vm.memory[0x3001] = 0x0000; // Null-terminated string
        println!("Registers before TRAP: {:?}", vm.registers);

        vm.trap_puts_p();

        println!("Registers after TRAP: {:?}", vm.registers);
        println!("Memory after TRAP: {:?}", &vm.memory[0x3000..0x3002]);
        assert_eq!(&vm.memory[0x3000..0x3002], &[0x4848, 0x0000]);
    }

    #[test]
    fn test_trap_halt() {
        let mut vm = VM::new();
        println!("Registers before TRAP: {:?}", vm.registers);

        vm.trap_halt();

        println!("Registers after TRAP: {:?}", vm.registers);
        assert!(!vm.running);
    }

    #[test]
    fn test_read_image() {
        let mut vm = VM::new();

        // Create a test file with appropriate data
        let mut file = File::create("test.obj").unwrap();
        let data: [u8; 6] = [
            0x30, 0x00, // Origin address in big-endian (0x3000)
            0x34, 0x12, // First data
            0x78, 0x56, // Second data
        ];
        file.write_all(&data).unwrap();

        // Read the image file and load the data into VM memory
        vm.read_image("test.obj").expect("Failed to read image");

        // Verify that the data was correctly loaded into memory
        assert_eq!(vm.memory[0x3000], 0x3412);
        assert_eq!(vm.memory[0x3001], 0x7856);
    }

    #[test]
    fn test_mem_write() {
        let mut vm = VM::new();
        // Set initial value for the memory
        vm.memory[0x3000] = 0x1234;
        println!("Memory before write: {:?}", &vm.memory[0x3000..0x3001]);

        vm.mem_write(0x3000, 0x5678);

        println!("Memory after write: {:?}", &vm.memory[0x3000..0x3001]);
        assert_eq!(vm.memory[0x3000], 0x5678);
    }

    // #[test]
    // fn test_mem_read_kbsr() {
    //     let mut vm = VM::new();
    //     // Set initial value for the memory
    //     vm.memory[usize::from(MemoryMappedRegister::Kbsr)] = 0x8000;
    //     println!(
    //         "Memory before read: {:?}",
    //         &vm.memory[MemoryMappedRegister::Kbsr.into()..]
    //     );

    //     let value = vm.mem_read(MemoryMappedRegister::Kbsr.into());

    //     println!("Value after read: {:?}", value);
    //     assert_eq!(value, 0x8000);
    // }

    // #[test]
    // fn test_mem_read_kbddr() {
    //     let mut vm = VM::new();
    //     // Set initial value for the memory
    //     vm.memory[usize::from(MemoryMappedRegister::Kbddr)] = 'a' as u16;
    //     println!(
    //         "Memory before read: {:?}",
    //         &vm.memory[MemoryMappedRegister::Kbddr.into()..]
    //     );

    //     let value = vm.mem_read(MemoryMappedRegister::Kbddr.into());

    //     println!("Value after read: {:?}", value);
    //     assert_eq!(value, 'a' as u16);
    // }

    #[test]
    fn test_mem_read() {
        let mut vm = VM::new();
        // Set initial value for the memory
        vm.memory[0x3000] = 0x1234;
        println!("Memory before read: {:?}", &vm.memory[0x3000..0x3001]);

        let value = vm.mem_read(0x3000);

        println!("Value after read: {:?}", value);
        assert_eq!(value, 0x1234);
    }

    // #[test]
    // fn test_check_key() {
    //     let vm = VM::new();
    //     let result = vm.check_key();
    //     println!("Result: {:?}", result);
    //     assert!(!result);
    // }
}
