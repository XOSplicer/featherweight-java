#![allow(dead_code)]

use crate::evaluation::eval_full;
use anyhow;
use std::path::PathBuf;

mod ast;
mod parser;
mod class_table;
mod evaluation;
mod typecheck;


#[derive(structopt::StructOpt)]
struct Args {
    #[structopt(parse(from_os_str))]
    fj_lib_file: PathBuf,
    #[structopt(parse(from_os_str))]
    fj_expression_file: PathBuf,
}

#[paw::main]
fn main(args: Args) -> anyhow::Result<()> {
    let input = std::fs::read_to_string(args.fj_lib_file).expect("could not read file");
    let ast = parser::parse(&input).expect("parsing failed");
    println!("AST {:#?}", &ast);
    let ct = class_table::ClassTable::try_from_ast(ast).expect("could not build class table");
    println!("CT {:#?}", &ct);

    let subtypes_of_object = ct.subtypes(&ast::ClassName("Object".into())).unwrap().cloned().collect::<Vec<_>>();
    println!("subtypes of object: {:?}", &subtypes_of_object);

    let input = std::fs::read_to_string(args.fj_expression_file).expect("could not read file");
    let term = parser::parse_eval_input(&input).expect("parsing failed");
    println!("TERM1 {}", &term);

    let result = eval_full(&ct, term).expect("eval failed");
    println!("RESULT1 {}", &result);

    Ok(())
}

