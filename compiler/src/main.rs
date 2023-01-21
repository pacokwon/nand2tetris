use std::fs::{self, File};
use std::path::Path;

use compiler::lexer::Lexer;
use compiler::parser::Parser;
use compiler::token::TokenType;
use compiler::xml_printer::XmlPrinter;

fn emit_tokens(filename: &str, input: &str) {
    let mut lexer = Lexer::new(input);
    let tokens: Vec<compiler::token::TokenType> = lexer
        .all_tokens()
        .into_iter()
        .map(|t| t.token_type)
        .collect::<Vec<TokenType>>();
    let mut output_file = File::create(filename).unwrap();
    tokens.print_xml(&mut output_file);
}

fn emit_ast(filename: &str, input: &str) {
    let mut parser = Parser::new(&input);
    let parse_tree = parser.parse();
    let mut file = File::create(filename).unwrap();
    parse_tree.print_xml(&mut file);
}

fn emit_syntax_analysis() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() < 2 {
        println!("help: compiler <path to jack code>");
        panic!("Please supply file name.");
    }

    if !args[1].ends_with(".jack") {
        panic!("Please supply a .jack file");
    }
    let filename_no_ext = Path::new(&args[1])
        .file_stem()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();
    let input = fs::read_to_string(&args[1])
        .expect("Expected path to valid jack file. Make sure the file exists.");

    let tokens_output = format!("{}T.xml", filename_no_ext);
    let ast_output = format!("{}.xml", filename_no_ext);

    emit_tokens(&tokens_output, &input);
    emit_ast(&ast_output, &input);
}

fn main() {
}
