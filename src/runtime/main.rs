extern crate im;
use im::hashmap::HashMap;

use super::super::parser::expressions::{Expression, Expression::*};

use std::rc::Rc;
use std::boxed::Box;

#[derive(Debug, Clone)]
struct LambdaFunction {
    env: Rc<Env>,
    arg_names: Vec<String>,
    body: Rc<Expression>,
    own_name: Option<String>
}

impl LambdaFunction {
    fn new_anonymous(env: Rc<Env>, arg_names: Vec<String>, body: Rc<Expression>) -> Self {
        return LambdaFunction {
            env: env,
            arg_names: arg_names,
            body: body,
            own_name: None
        }
    }
    fn new_named(env: Rc<Env>, name: String, arg_names: Vec<String>, body: Rc<Expression>) -> Self {
        LambdaFunction {
            env: env,
            arg_names: arg_names,
            body: body,
            own_name: Some(name)
        }
    }
    fn eval(self: Rc<Self>, arguments: Vec<Rc<Value>>) -> Rc<Value> {
        let mut new_vars = HashMap::new();
        self.arg_names.iter()
            .zip(arguments.into_iter())
            .for_each(|(name, value)|
                match new_vars.insert(name.clone(), value) {
                    _ => ()
                });
        match &self.own_name {
            Some(name) => {
                new_vars.insert(name.to_string(), Rc::new(Value::Lambda(self.clone())));
            },
            None => {}
        };
        let subenv = Rc::new(self.env.subenv(new_vars));
        return subenv.eval(&self.body).1
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeFunctionWrapper {
    Immediate(fn (Rc<Env>, Vec<Rc<Value>>) -> (Rc<Env>, Rc<Value>)),
    Symbolic(fn (Rc<Env>, Vec<Box<Expression>>) -> (Rc<Env>, Rc<Value>))
}

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    Lambda(Rc<LambdaFunction>),
    RuntimeFunction(RuntimeFunctionWrapper),
    Int(i32),
    Float(f32),
    Str(String)
}

#[derive(Debug, Clone)]
pub struct Env {
    table: HashMap<String, Rc<Value>>
}

impl Env {
    pub fn from_table(table: HashMap<String, Rc<Value>>) -> Self {
        println!("New env from table {:?}", table.keys().collect::<Vec<_>>());
        Env {
            table: table
        }
    }
    pub fn new() -> Self {
        println!("New empty env");
        Env {
            table: HashMap::new()
        }
    }
    fn subenv(&self, new_vars: HashMap<String, Rc<Value>>) -> Self {
        println!("New subenv {:?}", new_vars.keys().collect::<Vec<_>>());
        Env {
            table: new_vars.union(self.table.clone())
        }
    }
    fn add_name(self: &Self, name: String, value: Rc<Value>) -> Self {
        Env {
            table: self.table.update(name, value)
        }
    }
    pub fn lookup(&self, name: &String) -> Rc<Value> {
        self.table.get(name)
            .map_or_else(
                || Rc::new(Value::Nil),
                |val| val.clone())
    }
    pub fn eval(self: Rc<Self>, expr: &Expression) -> (Rc<Self>, Rc<Value>) {
        return match expr {
            ListExpr(contents) => self.eval_list(contents.to_vec()),
            SExpr(rator, rands) => self.eval_sexpr(rator.clone(), rands.to_vec()),
            LetExpr(name, rhs) => self.eval_let(name.to_string(), rhs),
            LambdaExpr(arg_list, body) => self.eval_lambda(arg_list.to_vec(), body.clone()),
            LookupExpr(name) => {
                let val = self.lookup(name);
                (self, val)
            },
            IntegerLiteral(v) => (self, Rc::new(Value::Int(*v))),
            FloatLiteral(v) => (self, Rc::new(Value::Float(*v))),
            StringLiteral(v) => (self, Rc::new(Value::Str(v.to_string())))
        }
    }
    fn eval_list(self: Rc<Self>, contents: Vec<Box<Expression>>) -> (Rc<Self>, Rc<Value>) {
        if contents.len() == 0 {
            return (self, Rc::new(Value::Nil));
        }
        let cons = self.lookup(&String::from("cons"));
        return (self, cons);
    }
    fn eval_sexpr(self: Rc<Self>, rator: Box<Expression>, rands: Vec<Box<Expression>>) -> (Rc<Self>, Rc<Value>) {
        let (mut env, func) = match &*rator {
            expr @ SExpr(_, _) |
            expr @ LookupExpr(_) |
            expr @ LambdaExpr(_, _) |
            expr @ LetExpr(_, _) => self.clone().eval(expr),
            _ => panic!("Cannot evaluate rator for s-expr")
        };
        return match &*func {
            Value::Lambda(lambda) => {
                let arguments = rands.into_iter().map(|a| {
                    let res = env.clone().eval(&a);
                    match res {
                        (new_env, val) => {
                            env = new_env;
                            return val;
                        }
                    }
                }).collect();
                (self, lambda.clone().eval(arguments))
            },
            Value::RuntimeFunction(RuntimeFunctionWrapper::Symbolic(internal)) => internal(self, rands),
            Value::RuntimeFunction(RuntimeFunctionWrapper::Immediate(internal)) => {
                let arguments = rands.into_iter().map(|a| {
                    let res = env.clone().eval(&a);
                    match res {
                        (new_env, val) => {
                            env = new_env;
                            return val;
                        }
                    }
                }).collect();
                internal(self, arguments)
            }
            _ => panic!("Bad rator for s-expr: {:?} ({:?})", func, rator)
        }
    }
    fn eval_let(self: Rc<Self>, name: String, rhs: &Expression) -> (Rc<Env>, Rc<Value>) {
        match rhs {
            LambdaExpr(arg_list, body) => {
                let lambda = LambdaFunction::new_named(self.clone(), name.clone(), arg_list.to_vec(), body.clone());
                let value = Rc::new(Value::Lambda(Rc::new(lambda)));
                return (Rc::new(self.add_name(name, value)), Rc::new(Value::Nil));
            }
            _  => {
                let (env, value) = self.clone().eval(rhs);
                return (Rc::new(env.add_name(name, value)), Rc::new(Value::Nil));
            }
        }
    }
    fn eval_lambda(self: Rc<Self>, arg_list: Vec<String>, body: Rc<Expression>) -> (Rc<Env>, Rc<Value>) {
        let lambda = Rc::new(Value::Lambda(Rc::new(LambdaFunction::new_anonymous(self.clone(), arg_list, body))));
        return (self, lambda);
    }
}
