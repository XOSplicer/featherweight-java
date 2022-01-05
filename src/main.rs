#![allow(dead_code)]

use crate::evaluation::eval_full;
use anyhow;
use anyhow::Context;
use std::path::PathBuf;
use typecheck::{typecheck_ast, typecheck_term, Gamma};

mod ast;
mod class_table;
mod class_tree;
mod error;
mod evaluation;
mod parser;
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
    let input = std::fs::read_to_string(args.fj_lib_file).context("could not read file")?;
    let ast = parser::parse(&input).context("parsing failed")?;
    println!("LIBRARY AST PARSED OK");
    let ct = class_table::ClassTable::try_from_ast(ast.clone())
        .context("could not build class table")?;
    println!("CLASS TABLE OK");

    typecheck_ast(&ct, &ast)?;
    println!("TYPECHECK for library OK");

    let subtypes_of_object = ct
        .subtypes(&ast::ClassName("Object".into()))
        .unwrap()
        .cloned()
        .collect::<Vec<_>>();
    println!("Subtypes of object: {:?}", &subtypes_of_object);

    let tree = class_tree::ClassTree::new(&ct);
    println!("CLASS TREE:\n {}", &tree);

    let input = std::fs::read_to_string(args.fj_expression_file).context("could not read file")?;
    let term = parser::parse_eval_input(&input).context("parsing failed")?;
    println!("TERM PARSED OK");
    println!("INPUT TERM {}", &term);

    let term_type = typecheck_term(&ct, &Gamma::empty(), &term)
        .context("Typechecking for input term failed")?;
    println!("TYPECHECK types term as {}", &term_type);

    let result = eval_full(&ct, term).context("eval failed")?;
    println!("EVALUATION RESULT {}", &result);

    Ok(())
}
