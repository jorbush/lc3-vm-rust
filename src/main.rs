const MEMORY_SIZE: usize = 65536; /* 65536 locations */

struct VM {
    memory: [u16; MEMORY_SIZE],
    registers: [u16; 10], /* 10 registers: R0-R7, COND and COUNT */
    pc: u16,              /* program counter */
}

impl VM {
    fn new() -> Self {
        VM {
            memory: [0; MEMORY_SIZE],
            registers: [0; 10],
            pc: 0,
        }
    }
}

fn main() {
    println!("Hello, world!");
}
