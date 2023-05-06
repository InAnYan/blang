use std::error::Error;

use file::{read_file, File};
use parser::Parser;
use scanner::Scanner;
use simple_compiler::Compiler;
use validator::Validator;
use std::rc::Rc;

mod ast;
mod file;
mod token;
mod scanner;
mod parser;
mod error_reporter;
mod validator;
mod simple_compiler;

pub struct Config {
    input_path: String,
    output_path: String,
    use_simple_compiler: bool
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("wrong arguments count")
        }

        let input_path = args[1].clone();
        let output_path = args[2].clone();
        let use_simple_compiler = !args.contains(&String::from("--tac"));

        Ok(Config {
            input_path,
            output_path,
            use_simple_compiler
        })
    }
}

// If there was a parsing error or semantic error, then this function will return Ok(()).
pub fn run(conf: &Config) -> Result<(), Box<dyn Error>> {
    let file = read_file(&conf.input_path)?;

    if conf.use_simple_compiler {
        run_simple_compiler(conf, file)
    } else {
        run_tac_compiler(conf, file)
    }
}

fn run_simple_compiler(conf: &Config, file: Rc<File>) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(file);
    let mut parser = Parser::new(&mut scanner);

    let mut validator = Validator::new();
    let mut compiler = Compiler::new();

    while !parser.is_at_end() {
        if let Some(decl) = parser.parse_one_decl() {
            if validator.validate_one_decl(&decl) {
                compiler.compile_one_decl(&decl)
            }
        }
    }

    match std::fs::write(&conf.output_path, compiler.get_code()) {
        Ok(()) => Ok(()),
        Err(e) => Err(Box::new(e))
    }
}

fn run_tac_compiler(_conf: &Config, _file: Rc<File>) -> Result<(), Box<dyn Error>> {
    Err("TAC compiler is under development".into())
}
