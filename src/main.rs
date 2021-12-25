#![allow(dead_code)]

use crate::evaluation::eval_full;

mod ast;
mod parser;
mod class_table;
mod evaluation;

fn main() {
    let input = std::fs::read_to_string("test.fj").expect("could not read file");
    let ast = parser::parse(&input).expect("parsing failed");
    println!("AST {:#?}", &ast);
    let ct = class_table::ClassTable::try_from_ast(ast).expect("could not build class table");
    println!("CT {:#?}", &ct);
    let triple_fields = ct
        .fields(&ast::ClassName("Triple".into()))
        .unwrap()
        .collect::<Vec<_>>();
    println!("fields of `Triple`: {:?}", &triple_fields);
    let mtype_triple_setfst = ct.method_type(&ast::MethodName("setfst".into()), &ast::ClassName("Triple".into()));
    println!("type of of `Triple::setfst`: {:?}", &mtype_triple_setfst);

    let supertypes_of_triple = ct.super_type_chain(&ast::ClassName("Triple".into())).unwrap().cloned().collect::<Vec<_>>();
    println!("supertypes of triple: {:?}", &supertypes_of_triple);

    let subtypes_of_pair = ct.subtypes(&ast::ClassName("Pair".into())).unwrap().cloned().collect::<Vec<_>>();
    println!("subtypes of pair: {:?}", &subtypes_of_pair);

    let subtypes_of_object = ct.subtypes(&ast::ClassName("Object".into())).unwrap().cloned().collect::<Vec<_>>();
    println!("subtypes of object: {:?}", &subtypes_of_object);


    let input = std::fs::read_to_string("test.fje").expect("could not read file");
    let term = parser::parse_eval_input(&input).expect("parsing failed");
    println!("TERM1 {}", &term);

    let result = eval_full(&ct, term).expect("eval failed");
    println!("RESULT1 {}", &result);

    let input = std::fs::read_to_string("test2.fje").expect("could not read file");
    let term = parser::parse_eval_input(&input).expect("parsing failed");
    println!("TERM2 {}", &term);

    let result = eval_full(&ct, term).expect("eval failed");
    println!("RESULT2 {}", &result);

}

