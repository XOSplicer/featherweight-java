#![allow(dead_code)]

mod ast;
mod parser;
mod semantic;

fn main() {
    let input = std::fs::read_to_string("test.fj").expect("could not read file");
    let ast = parser::parse(&input).expect("parsing failed");
    println!("AST {:#?}", &ast);
    let ct = semantic::ClassTable::try_from_ast(ast).expect("could not build class table");
    println!("CT {:#?}", &ct);
    let triple_fields = ct
        .fields(&ast::ClassName("Triple".into()))
        .unwrap()
        .collect::<Vec<_>>();
    println!("fields of `Triple`: {:?}", &triple_fields);
    let mtype_triple_setfst = ct.method_type(&ast::MethodName("setfst".into()), &ast::ClassName("Triple".into()));
    println!("type of of `Triple::setfst`: {:?}", &mtype_triple_setfst);
}
