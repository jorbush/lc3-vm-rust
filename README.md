# LC-3 Virtual Machine in Rust

This project is an implementation of the LC-3 Virtual Machine (VM) in Rust.

## What is LC-3?

The LC-3 is a simple educational computer designed to teach the basics of computer architecture and machine language programming.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install)

## Implemented Instructions

| Instruction | OpCode           | Description                                                         | Implemented  |
|-------------|------------------|---------------------------------------------------------------------|--------------|
| BR          | 0000             | Performs a conditional branch based on the condition flags.         | ✅           |
| ADD         | 0001             | Adds two values. Can use immediate mode or register mode.           | ✅           |
| LD          | 0010             | Loads a value from a memory address into a register.                | ✅           |
| ST          | 0011             | Stores the value of a register into a memory address.               | ✅           |
| JSR         | 0100             | Jumps to a subroutine and saves the return address.                 | ✅           |
| AND         | 0101             | Performs a bitwise AND operation between two registers.             | ✅           |
| LDR         | 0110             | Loads a value from a memory address calculated based on a register. | ✅           |
| STR         | 0111             | Stores the value of a register into a calculated memory address.    | ✅           |
| RTI         | 1000             | Unused.                                                             | ✅           |
| NOT         | 1001             | Performs a bitwise NOT operation on a register.                     | ✅           |
| LDI         | 1010             | Loads a value from an indirect memory address into a register.      | ✅           |
| STI         | 1011             | Stores the value of a register into an indirect memory address.     | ✅           |
| JMP         | 1100             | Jumps to the address contained in a register.                       | ✅           |
| RES         | 1101             | Unused.                                                             | ✅           |
| LEA         | 1110             | Loads the effective address into a register.                        | ✅           |
| TRAP        | 1111             | Invokes an operating system routine.                                | ✅           |

## Usage

To run the virtual machine, use the following command:

```bash
cargo run examples/2048.obj
```

## Testing

To run the tests, use the following command:

```bash
cargo test vm
```

## Formatting

To format the code, use the following command:

```bash
cargo fmt
```

## References

This project is based on the following articles and resources:
- [The tutorial by James Meiners](https://www.jmeiners.com/lc3-vm/)
- [Specification](https://www.jmeiners.com/lc3-vm/supplies/lc3-isa.pdf)
