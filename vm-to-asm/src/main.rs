use std::fs;

use vm_to_asm::{code_writer::CodeWriter, command::CommandType, parser::Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        println!("help: vm-to-asm <filename>");
        panic!("Please input filename as argument.");
    }

    let input = fs::read_to_string(&args[1]).expect("Source file not found.");
    let mut parser = Parser::new(&input);
    let mut code_writer = CodeWriter::new();
    code_writer.set_filename("output.asm");

    while parser.has_more_commands() {
        let command_type = parser.command_type();
        match command_type {
            CommandType::Arithmetic => {
                let command = parser.arg1();
                code_writer.write_arithemtic(command);
            }
            CommandType::Push | CommandType::Pop => {
                let segment = parser.arg1();
                let index = parser.arg2();
                code_writer.write_pushpop(command_type, segment, index);
            }
            _ => todo!(),
        }

        parser.advance();
    }
}
