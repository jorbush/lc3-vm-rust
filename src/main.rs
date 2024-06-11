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

    for arg in &args[1..] {
        if let Err(e) = vm.load_image(arg) {
            eprintln!("failed to load image: {}: {}", arg, e);
            std::process::exit(1);
        }
    }

    vm.run();
}
