extern crate termios;

use libc::STDIN_FILENO;
use signal_hook::{iterator::Signals, SIGINT};
use std::process;
use std::{error::Error, thread};
use termios::*;

fn handle_control_c(_sig: i32) {
    restore_terminal_settings();
    println!("\n\n");
    println!("The LC3 VM received Ctrl-C interrupt signal.");
    process::exit(130);
}

pub fn restore_terminal_settings() {
    let mut term: Termios = Termios::from_fd(STDIN_FILENO).unwrap();
    term.c_lflag |= ICANON | ECHO;
    tcsetattr(STDIN_FILENO, TCSANOW, &term).unwrap();
}

pub fn turn_off_canonical_and_echo_modes() {
    let mut term: Termios = Termios::from_fd(STDIN_FILENO).unwrap();
    term.c_lflag &= !(ICANON | ECHO);
    tcsetattr(STDIN_FILENO, TCSANOW, &term).unwrap();
}

pub fn spawn_control_c_handler() -> Result<(), Box<dyn Error>> {
    let signals = Signals::new(&[SIGINT])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            handle_control_c(sig);
        }
    });
    Ok(())
}
