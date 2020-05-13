extern crate im;
use im::hashmap::HashMap;
use super::main::{Env, Value, RuntimeFunctionWrapper};
use super::super::parser::expressions::Expression;
use super::super::parser::main::Parser;

use either::*;
use std::rc::Rc;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use std::process::exit;

type IFold = fn (i32, i32) -> i32;
type FFold = fn (f32, f32) -> f32;

fn try_fold_ints(mut args: Vec<Rc<Value>>, i_base: i32, i_fold: IFold, f_base: f32, f_fold: FFold) -> Either<i32, f32> {
    match args.pop() {
        Some(v) => match *v {
            Value::Int(num) => match try_fold_ints(args, i_base, i_fold, f_base, f_fold) {
                Left(i) => Left(i_fold(num, i)),
                Right(f) => Right(f_fold(num as f32, f))
            },
            Value::Float(num) => Right(f_fold(num, fold_floats(args, f_base, f_fold))),
            _ => panic!("Bad values in built-in")
        },
        None => Left(i_base)
    }
}

fn fold_floats(mut args: Vec<Rc<Value>>, f_base: f32, f_fold: FFold) -> f32 {
    match args.pop() {
        Some(v) => match *v {
            Value::Int(num) => f_fold(num as f32, fold_floats(args, f_base, f_fold)),
            Value::Float(num) => f_fold(num, fold_floats(args, f_base, f_fold)),
            _ => panic!("Bad values in add call")
        },
        None => f_base
    }
}

fn add_i(a: i32, b: i32) -> i32 {
    a + b
}
fn add_f(a: f32, b: f32) -> f32 {
    a + b
}

fn sub_i(a: i32, b: i32) -> i32 {
    a - b
}
fn sub_f(a: f32, b: f32) -> f32 {
    a - b
}

fn mul_i(a: i32, b: i32) -> i32 {
    a * b
}
fn mul_f(a: f32, b: f32) -> f32 {
    a * b
}

fn div_i(a: i32, b: i32) -> i32 {
    a / b
}
fn div_f(a: f32, b: f32) -> f32 {
    a / b
}

fn wrapped_add(env: Rc<Env>, mut args: Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>) {
    (env, Rc::new(match try_fold_ints(args, 0, add_i, 0.0, add_f) {
        Left(i) => Value::Int(i),
        Right(f) => Value::Float(f)
    }))
}

fn wrapped_sub(env: Rc<Env>, mut args: Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>) {
    (env, Rc::new(match try_fold_ints(args, 0, sub_i, 0.0, sub_f) {
        Left(i) => Value::Int(i),
        Right(f) => Value::Float(f)
    }))
}

fn wrapped_mul(env: Rc<Env>, mut args: Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>) {
    (env, Rc::new(match try_fold_ints(args, 1, mul_i, 1.0, mul_f) {
        Left(i) => Value::Int(i),
        Right(f) => Value::Float(f)
    }))
}

fn wrapped_div(env: Rc<Env>, mut args: Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>) {
    (env, Rc::new(match try_fold_ints(args, 1, div_i, 1.0, div_f) {
        Left(i) => Value::Int(i),
        Right(f) => Value::Float(f)
    }))
}

fn wrapped_exit(env: Rc<Env>, mut args: Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>) {
    exit(0);
    (env, Rc::new(Value::Nil))
}


fn fn_true(env: Rc<Env>, mut args: Vec<Box<Expression>>) -> (Rc<Env>, Rc<Value>) {
    let (env, res) = env.eval(&args[0]);
    (env, res)
}

fn fn_false(env: Rc<Env>, mut args: Vec<Box<Expression>>) -> (Rc<Env>, Rc<Value>) {
    let (env, res) = env.eval(&args[1]);
    (env, res)
}

fn fn_eq(env: Rc<Env>, mut args: Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>) {
    if args.len() != 2 {
        panic!("Malformed eq predicate");
    }
    let x = args.pop().unwrap();
    let y = args.pop().unwrap();
    let res = match &*x {
        Value::Nil => match &*y {
            Value::Nil => env.lookup(&String::from("true")),
            _ => env.lookup(&String::from("false"))
        },
        Value::Int(a) => match &*y {
            Value::Int(b) => if a == b {
                env.lookup(&String::from("true"))
            } else {
                env.lookup(&String::from("false"))
            },
            _ => env.lookup(&String::from("false"))
        },
        Value::Float(a) => match &*y {
            Value::Float(b) => if a == b {
                env.lookup(&String::from("true"))
            } else {
                env.lookup(&String::from("false"))
            },
            _ => env.lookup(&String::from("false"))
        },
        Value::Str(a) => match &*y {
            Value::Str(b) => if a == b {
                env.lookup(&String::from("true"))
            } else {
                env.lookup(&String::from("false"))
            },
            _ => env.lookup(&String::from("false"))
        },
        _ => env.lookup(&String::from("false"))
    };
    return (env, res);
}

pub fn build_standard_library() -> Rc<Env> {
    // load in runtime builtins

    let mut table = HashMap::new();
    table.insert(String::from("+"), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Immediate(wrapped_add))));
    table.insert(String::from("-"), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Immediate(wrapped_sub))));
    table.insert(String::from("*"), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Immediate(wrapped_mul))));
    table.insert(String::from("/"), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Immediate(wrapped_div))));
    table.insert(String::from("="), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Immediate(fn_eq))));
    table.insert(String::from("exit"), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Immediate(wrapped_exit))));
    table.insert(String::from("true"), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Symbolic(fn_true))));
    table.insert(String::from("false"), Rc::new(Value::RuntimeFunction(RuntimeFunctionWrapper::Symbolic(fn_false))));
    table.insert(String::from("nil"), Rc::new(Value::Nil));

    // load our standard libray

    let path = Path::new("core.scm");
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(),
                                                   why.description()),
        Ok(file) => file,
    };
    let mut env = Rc::new(Env::from_table(table));
    let mut buffer = String::new();

    match file.read_to_string(&mut buffer) {
        Err(why) => panic!("couldn't read {}: {}", path.display(),
                                                   why.description()),
        Ok(_) => {}
    };

    let mut p = Parser::new(&buffer, 0);
    while !p.is_finished() {
        let tree = p.parse(false).unwrap();
        match tree {
            Either::Left(expr) => {
                match env.eval(&expr) {
                    (new_env, res) => {
                        env = new_env;
                    }
                }
                
            },
            Either::Right(_) => {}
        }
    }
    return env;
}
