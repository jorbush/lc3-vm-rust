mod utils;
mod vm;

use std::env;
use vm::VM;

fn main() {
    utils::terminal::spawn_control_c_handler().unwrap();

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

    utils::terminal::restore_terminal_settings();
    println!("Shutting Down VM...");
}
