use crate::ast;
use anyhow::Result;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "fj.pest"]
pub struct FJParser;

pub fn parse(input: &str) -> Result<ast::Ast> {
    let program = FJParser::parse(Rule::program, input)?;
    parse_program(program)
}

fn parse_program(pairs: Pairs<Rule>) -> Result<ast::Ast> {
    println!("parse_program {:#?}", &pairs);
    let class_definitions = pairs
        .take_while(|pair| pair.as_rule() != Rule::EOI)
        .map(parse_class_definition)
        .collect::<Result<Vec<_>>>()?;
    Ok(ast::Ast { class_definitions })
}

fn parse_class_definition(pair: Pair<Rule>) -> Result<ast::ClassDefinition> {
    println!("parse_class_definition {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::class_definition => {
            let mut pairs = pair.into_inner();
            let name = pairs.next().unwrap().as_str();
            let super_type = pairs.next().unwrap().as_str();

            let fields = {
                pairs
                    .clone()
                    .take_while(|pair| pair.as_rule() == Rule::field_definition)
                    .map(parse_field_definition)
                    .collect::<Result<Vec<_>>>()?
            };
            let mut pairs = pairs.skip_while(|pair| pair.as_rule() == Rule::field_definition);
            let ctor_pair = pairs.next().unwrap();
            let constructor = parse_constructor(ctor_pair)?;
            let methods = pairs
                .map(parse_method_definition)
                .collect::<Result<Vec<_>>>()?;
            ast::ClassDefinition {
                name: ast::ClassName(name.into()),
                super_type: ast::ClassName(super_type.into()),
                fields,
                constructor,
                methods,
            }
        }
        _ => unreachable!(),
    })
}

fn parse_field_definition(pair: Pair<Rule>) -> Result<ast::ArgPair> {
    println!("parse_field_definition {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::field_definition => {
            let mut pairs = pair.into_inner();
            let class_name = pairs.next().unwrap().as_str();
            let field_name = pairs.next().unwrap().as_str();
            (
                ast::ClassName(class_name.into()),
                ast::FieldName(field_name.into()),
            )
        }
        _ => unreachable!(),
    })
}

fn parse_constructor(pair: Pair<Rule>) -> Result<ast::Constructor> {
    println!("parse_constructor {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::constructor => {
            let mut pairs = pair.into_inner();
            let name = pairs.next().unwrap().as_str();
            let args = pairs
                .clone()
                .find(|pair| pair.as_rule() == Rule::arg_list)
                .map(parse_arg_list)
                .transpose()?
                .unwrap_or_default();
            let super_call = pairs
                .clone()
                .find(|pair| pair.as_rule() == Rule::field_list)
                .map(parse_super_field_list)
                .transpose()?
                .unwrap_or_default();
            let assignments = pairs
                .clone()
                .filter(|pair| pair.as_rule() == Rule::assignment)
                .map(parse_assignment)
                .collect::<Result<Vec<_>>>()?;
            ast::Constructor {
                name: ast::ClassName(name.into()),
                args,
                super_call,
                assignments,
            }
        }
        _ => unreachable!(),
    })
}

fn parse_arg_list(pair: Pair<Rule>) -> Result<Vec<ast::ArgPair>> {
    println!("parse_arg_list {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::arg_list => {
            let mut pairs = pair.into_inner().peekable();
            let mut args = Vec::new();
            while pairs.peek().is_some() {
                args.push((
                    ast::ClassName(pairs.next().unwrap().as_str().into()),
                    ast::FieldName(pairs.next().unwrap().as_str().into()),
                ));
            }
            args
        }
        _ => unreachable!(),
    })
}

fn parse_super_field_list(pair: Pair<Rule>) -> Result<Vec<ast::FieldName>> {
    println!("parse_super_field_list {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::field_list => {
            let pairs = pair.into_inner();
            pairs
                .map(|pair| ast::FieldName(pair.as_str().into()))
                .collect()
        }
        _ => unreachable!(),
    })
}

fn parse_assignment(pair: Pair<Rule>) -> Result<(ast::FieldName, ast::FieldName)> {
    println!("parse_assignment {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::assignment => {
            let mut pairs = pair.into_inner();
            (
                ast::FieldName(pairs.next().unwrap().as_str().into()),
                ast::FieldName(pairs.next().unwrap().as_str().into()),
            )
        }
        _ => unreachable!(),
    })
}

fn parse_method_definition(pair: Pair<Rule>) -> Result<ast::MethodDefinition> {
    println!("parse_method_definition {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::method_definition => {
            let mut pairs = pair.into_inner();
            let return_type = pairs.next().unwrap().as_str();
            let method_name = pairs.next().unwrap().as_str();
            let args = pairs
                .clone()
                .find(|pair| pair.as_rule() == Rule::arg_list)
                .map(parse_arg_list)
                .transpose()?
                .unwrap_or_default();
            let return_term = pairs
                .clone()
                .find(|pair| pair.as_rule() == Rule::term)
                .map(parse_term)
                .transpose()?
                .unwrap()
                .boxed();
            ast::MethodDefinition {
                return_type: ast::ClassName(return_type.into()),
                method_name: ast::MethodName(method_name.into()),
                args,
                return_term,
            }
        }
        _ => unreachable!(),
    })
}

fn parse_term(pair: Pair<Rule>) -> Result<ast::Term> {
    println!("parse_term {:#?}", &pair);
    Ok(match pair.as_rule() {
        Rule::term => {
            let mut pairs = pair.into_inner();
            todo!()
        }
        _ => unreachable!(),
    })
}
