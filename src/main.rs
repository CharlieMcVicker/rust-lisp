extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use std::rc::Rc;
use std::mem::size_of;
use either::*;

mod lexer;
mod parser;
mod runtime;
use parser::main::Parser;

fn main() {
    let mut rl = Editor::<()>::new();
    let mut env = runtime::stdlib::build_standard_library();
    // let mut env = Rc::new(runtime::main::Env::new());
    // println!("Size of env {}", size_of::<runtime::main::Env>());
    loop {
        match rl.readline("  > ") {
            Ok(buffer) => {
                let mut p = Parser::new(&buffer, 0);
                while !p.is_finished() {
                    match p.parse(false) {
                        Ok(tree) => match tree {
                            Either::Left(expr) => {
                                match env.eval(&expr) {
                                    (new_env, res) => {
                                        println!("{:?}", res);
                                        env = new_env;
                                    }
                                }
                                
                            },
                            Either::Right(_) => {}
                        },
                        Err(err) => println!("{:?}", err)
                    };
                }
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
