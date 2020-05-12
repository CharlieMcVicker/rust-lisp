extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::io::{stdin, stdout};

mod lexer;
mod parser;
mod runtime;
use parser::main::Parser;

fn main() {
    let mut rl = Editor::<()>::new();
    loop {
        match rl.readline("  > ") {
            Ok(buffer) => {
                let mut p = Parser::new(&buffer, 0);
                println!("{:?}", p.parse(false))
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}
