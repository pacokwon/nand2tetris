# Compiler

This is the compiler implementation as featured in Chapter 10, 11 of the book.

The implementation is written in Rust 1.65.

The compiler, given a single .jack file or a directory containing multiple .jack files, compiles the Jack source files into VM instructions.

The compiler features fundamental programming language features such as arithmetic operators, conditionals, loops, and function calls, classes and methods.

## Building
```bash
$ cargo build
```

## Running
```bash
$ cargo run <input .jack file or directory containing .jack files>
```

## Syntax Analysis Output in XML
This was the main task of chapter 9, but the completed program is a full compiler as described above.

To make the compiler emit syntax analysis results, simply change the `main` function in `src/main.rs` as follows:

```rust
fn main() {
    emit_syntax_analysis();
}
```

And run the program again.
