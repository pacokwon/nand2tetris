use std::fs;
use std::env;

use assembler::parser::Parser;
use assembler::pass::Pass;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        println!("help: assembler <filename>");
        panic!("Please input filename as argument.");
    }

    let input = fs::read_to_string(&args[1])?;
    let mut parser = Parser::new(&input);
    let instructions: Vec<assembler::instruction::Instruction> = parser.parse();
    let minsts = Pass::translate(&instructions);
    println!("{}", minsts.join("\n"));

    Ok(())
}
