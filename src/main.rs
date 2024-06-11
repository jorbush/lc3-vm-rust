mod vm;

use std::env;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("lc3 [image-file1] ...");
        std::process::exit(2);
    }

    let mut vm = VM::new();
}
