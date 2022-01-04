use std::collections::BTreeMap;

use crate::class_table::MethodType;
use crate::error::TypingError;
use crate::{ast::*, class_table::ClassTable};
use anyhow::{Context, Result};
use std::iter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gamma {
    inner: BTreeMap<FieldName, ClassName>,
}

impl Gamma {
    pub fn new() -> Self {
        Gamma {
            inner: BTreeMap::new(),
        }
    }
    pub fn empty() -> Self {
        Self::new()
    }
    pub fn from_class_method(class_name: &ClassName, method: &MethodDefinition) -> Self {
        Gamma {
            inner: iter::once((FieldName("this".into()), class_name.clone()))
                .chain(
                    method.args.iter().map(|(arg_class_name, arg_name)| {
                        (arg_name.clone(), arg_class_name.clone())
                    }),
                )
                .collect(),
        }
    }
}

pub fn typecheck_term(ct: &ClassTable, gamma: &Gamma, term: &Term) -> Result<ClassName> {
    match term {
        //T-Var
        Term::Variable(x) => gamma
            .inner
            .get(x)
            .cloned()
            .ok_or(TypingError::VariableNotInGamma(x.clone()).into()),
        // T-Field
        Term::FieldAccess(FieldAccess { field, object_term }) => {
            typecheck_term(ct, gamma, object_term).and_then(|object_term_type| {
                ct.fields(&object_term_type)
                    .ok_or(TypingError::UndefinedClass(object_term_type.clone()))?
                    .find(|(_, field_name)| field_name == field)
                    .map(|(class_name, _)| class_name)
                    .cloned()
                    .ok_or(
                        TypingError::UndefinedField(field.clone(), object_term_type.clone()).into(),
                    )
            })
        }
        // T-Invk
        Term::MethodCall(MethodCall {
            arg_terms,
            method_name,
            object_term,
        }) => {
            typecheck_term(ct, gamma, object_term).and_then(|object_term_type| {
                let method_type = ct.method_type(&method_name, &object_term_type).ok_or(
                    TypingError::UndefinedMethod(method_name.clone(), object_term_type.clone()),
                )?;
                let arg_term_types = arg_terms
                    .iter()
                    .map(|arg_term| typecheck_term(ct, gamma, arg_term))
                    .map(|r| r.map_err(From::from))
                    .collect::<Result<Vec<_>>>()?;
                // TODO: check matching length of both lists
                arg_term_types
                    .iter()
                    .zip(method_type.arg_types.iter())
                    .map(|(c, d)| match ct.is_subtype(c, d) {
                        Some(true) => Ok(()),
                        Some(false) => Err(TypingError::MethodArgumentNotSubtype(
                            c.clone(),
                            d.clone(),
                            method_name.clone(),
                            object_term_type.clone(),
                        )),
                        None => Err(TypingError::UndefinedClasses(vec![c.clone(), d.clone()])),
                    })
                    .map(|r| r.map_err(From::from))
                    .collect::<Result<()>>()?;
                Ok(method_type.ret_type)
            })
        }
        // T-New
        Term::NewCall(NewCall {
            class_name,
            arg_terms,
        }) => {
            let fields = ct
                .fields(class_name)
                .ok_or(TypingError::UndefinedClass(class_name.clone()))?;
            let arg_term_types = arg_terms
                .iter()
                .map(|arg_term| typecheck_term(ct, gamma, arg_term))
                .collect::<Result<Vec<_>>>()?;
            // TODO: check matching length of both lists
            arg_term_types
                .iter()
                .zip(fields.map(|(field_type, _)| field_type))
                .map(|(c, d)| match ct.is_subtype(c, d) {
                    Some(true) => Ok(()),
                    Some(false) => Err(TypingError::ConstructorArgumentNotSubtype(
                        c.clone(),
                        d.clone(),
                        class_name.clone(),
                    )),
                    None => Err(TypingError::UndefinedClasses(vec![c.clone(), d.clone()])),
                })
                .map(|r| r.map_err(From::from))
                .collect::<Result<()>>()?;
            Ok(class_name.clone())
        }
        Term::Cast(Cast {
            to_class_name,
            term,
        }) => {
            let term_type = typecheck_term(ct, gamma, term)?;
            if !ct.contains_class(to_class_name) {
                Err(TypingError::UndefinedClass(to_class_name.clone()))?;
            }
            if !ct.contains_class(&term_type) {
                Err(TypingError::UndefinedClass(term_type.clone()))?;
            }
            // T-UpCast
            // dbg!(&to_class_name);
            // dbg!(&term_type);

            if ct.is_subtype(&term_type, to_class_name).unwrap() {
                return Ok(to_class_name.clone());
            }
            // T-DownCast
            else if ct.is_subtype(to_class_name, &term_type).unwrap()
                && to_class_name != &term_type
            {
                return Ok(to_class_name.clone());
            }
            // T-SenselessCast
            else if !ct.is_subtype(&term_type, to_class_name).unwrap()
                && !ct.is_subtype(to_class_name, &term_type).unwrap()
            {
                // FIXME: should this be an actual error?
                println!(
                    "SENSELESS CAST WARNING: Term of type `{}` can not be cast to type `{}`",
                    &term_type, &to_class_name
                );
                return Ok(to_class_name.clone());
            }
            // cast fallthrough should not happen
            else {
                Err(TypingError::InvalidCast {
                    from: term_type.clone(),
                    to: to_class_name.clone(),
                })?
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodOk(/* in */ pub ClassName);

pub fn typecheck_method(
    ct: &ClassTable,
    method: &MethodDefinition,
    in_class_name: &ClassName,
) -> Result<MethodOk> {
    let gamma = Gamma::from_class_method(in_class_name, method);
    let ret_term_type = typecheck_term(ct, &gamma, &method.return_term)
        .context(TypingError::InvalidTerm(*method.return_term.clone()))?;
    let super_type = ct
        .super_type(in_class_name)
        .ok_or(TypingError::UndefinedClass(in_class_name.clone()))?;
    let method_type = MethodType::from_method(method);
    if !ct
        .is_correct_method_override(&method.method_name, in_class_name, &method_type)
        .ok_or(TypingError::UndefinedMethod(
            method.method_name.clone(),
            in_class_name.clone(),
        ))?
    {
        Err(TypingError::IncorrectMethodOverride(
            method.method_name.clone(),
            in_class_name.clone(),
        ))?;
    }
    if !ct.is_subtype(&ret_term_type, &method_type.ret_type).ok_or(
        anyhow::Error::from(TypingError::UndefinedClass(ret_term_type.clone())).context(
            TypingError::UndefinedReturnType(method.method_name.clone(), in_class_name.clone()),
        ),
    )? {
        Err(TypingError::ReturnTypeNotSubtype(
            ret_term_type.clone(),
            method_type.ret_type.clone(),
            method.method_name.clone(),
            in_class_name.clone(),
        ))?;
    }
    Ok(MethodOk(in_class_name.clone()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassOk;

pub fn typecheck_class(ct: &ClassTable, class: &ClassDefinition) -> Result<ClassOk> {
    // TODO: many checks are already done when building the class table. they might be moved here

    let super_type = ct
        .super_type(&class.name)
        .ok_or(TypingError::UndefinedClass(class.name.clone()))?;
    let super_fields = ct.fields(&super_type).unwrap();

    // TODO: check correct super() call

    let _ = class
        .methods
        .iter()
        .map(|method| {
            typecheck_method(ct, method, &class.name).context(TypingError::InvalidMethod(
                method.method_name.clone(),
                class.name.clone(),
            ))
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ClassOk)
}

pub fn typecheck_ast(ct: &ClassTable, ast: &Ast) -> Result<()> {
    ast.class_definitions
        .iter()
        .map(|class| {
            typecheck_class(ct, class).context(TypingError::InvalidClass(class.name.clone()))
        })
        .map(|r| r.map(|_| ()))
        .collect()
}
