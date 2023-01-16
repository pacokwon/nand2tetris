use std::{fs, path::Path};

use vm_to_asm::{code_writer::CodeWriter, command::CommandType, parser::Parser};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("help: vm-to-asm <input vm file> [output asm file]");
        panic!("Please provide input and output filenames.");
    }

    let filenames: Vec<String> = if args[1].ends_with(".vm") {
        vec![args[1].to_string()]
    } else {
        fs::read_dir(&args[1])
            .expect(&format!("{} is not a directory", args[1]))
            .map(|entry| {
                entry
                    .unwrap()
                    .path()
                    .into_os_string()
                    .into_string()
                    .unwrap()
            })
            .filter(|path| path.ends_with(".vm"))
            .collect::<Vec<String>>()
    };
    println!("{:?}", filenames);

    let mut code_writer = CodeWriter::new();
    let output = if args.len() < 3 {
        format!("{}.asm", &args[1][0..args[1].len() - 3])
    } else {
        args[2].clone()
    };

    code_writer.set_output_filename(&output);
    code_writer.write_init();

    for filepath in &filenames {
        let input = fs::read_to_string(filepath).expect("Source file not found.");
        let mut parser = Parser::new(&input);

        // `filepath` contains a full path. extract the filename only.
        let filename = Path::new(filepath).file_name().unwrap().to_str().unwrap();
        println!("{}", filename);

        code_writer.set_module_name(filename);
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
                CommandType::Label => {
                    let label = parser.arg1();
                    code_writer.write_label(label);
                }
                CommandType::Goto => {
                    let label = parser.arg1();
                    code_writer.write_goto(label);
                }
                CommandType::If => {
                    let label = parser.arg1();
                    code_writer.write_if(label);
                }
                CommandType::Function => {
                    let function_name = parser.arg1();
                    let locals_count = parser.arg2();
                    code_writer.write_function(function_name, locals_count);
                }
                CommandType::Call => {
                    let function_name = parser.arg1();
                    let arg_count = parser.arg2();
                    code_writer.write_call(function_name, arg_count);
                }
                CommandType::Return => {
                    code_writer.write_return();
                }
            }

            parser.advance();
        }
    }
}
