use std::{fs::File, io::Write};

pub trait XmlPrinter {
    fn print_xml(&self, file: &mut File);
}

pub fn print_opening(file: &mut File, tag: &str) {
    writeln!(file, "<{tag}>").unwrap();
}

pub fn print_closing(file: &mut File, tag: &str) {
    writeln!(file, "</{tag}>").unwrap();
}

pub fn print_tag(file: &mut File, tag: &str, content: &str) {
    writeln!(file, "<{tag}> {content} </{tag}>").unwrap();
}

pub fn print_symbol(file: &mut File, content: &str) {
    writeln!(file, "<symbol> {content} </symbol>").unwrap();
}

impl XmlPrinter for char {
    fn print_xml(&self, file: &mut File) {
        match self {
            '+' | '-' | '*' | '/' | '|' | '=' | '~' | '{' | '}' | '(' | ')' | '[' | ']' | ';'
            | '.' | ',' => {
                writeln!(file, "<symbol> {} </symbol>", self).unwrap();
            }
            '<' => writeln!(file, "<symbol> &lt; </symbol>").unwrap(),
            '>' => writeln!(file, "<symbol> &gt; </symbol>").unwrap(),
            '&' => writeln!(file, "<symbol> &amp; </symbol>").unwrap(),
            _ => panic!("{} is not a valid operator!", self),
        }
    }
}
