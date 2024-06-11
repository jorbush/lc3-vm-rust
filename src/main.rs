const MEMORY_SIZE: usize = 65536; /* 65536 locations */

struct VM {
    memory: [u16; MEMORY_SIZE],
    registers: [u16; 10],
    pc: u16,
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
