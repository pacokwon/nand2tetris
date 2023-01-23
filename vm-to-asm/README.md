# Vm to Asm Translator

This is the vm to assembly translator implementation in Chapter 7, 8 of the book.

The implementation is written in Rust 1.65.

This translator, given a single .vm file or a directory containing multiple .vm files, translates the vm instructions to Hack assembly instructions.

This stack-based virtual machine supports basic arithmetic operations, branching instructions, labels, and function calls.

## Building
```bash
$ cargo build
```

## Running
```bash
$ cargo run <input .vm file or directory containing .vm files> [output asm file name]
```
