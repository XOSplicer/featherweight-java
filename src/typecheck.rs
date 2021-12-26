use std::collections::BTreeMap;

use crate::class_table::MethodType;
use crate::{ast::*, class_table::ClassTable};
use anyhow::{anyhow, bail, Result};
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
            .ok_or(anyhow!("Field `{}` not typed by gamma", &x.0)),
        // T-Field
        Term::FieldAccess(FieldAccess { field, object_term }) => {
            typecheck_term(ct, gamma, object_term).and_then(|object_term_type| {
                ct.fields(&object_term_type)
                    .ok_or(anyhow!(
                        "Class `{}` not in class table",
                        &object_term_type.0
                    ))?
                    .find(|(_, field_name)| field_name == field)
                    .map(|(class_name, _)| class_name)
                    .cloned()
                    .ok_or(anyhow!(
                        "Field `{}` not present in class `{}`.",
                        &field.0,
                        &object_term_type.0
                    ))
            })
        }
        // T-Invk
        Term::MethodCall(MethodCall {
            arg_terms,
            method_name,
            object_term,
        }) => {
            typecheck_term(ct, gamma, object_term).and_then(|object_term_type| {
                let method_type = ct.method_type(&method_name, &object_term_type)
                    .ok_or(anyhow!(
                        "Method `{}` not present in class `{}`.",
                        &method_name.0,
                        &object_term_type.0
                    ))?;
                let arg_term_types = arg_terms.iter().map(|arg_term| typecheck_term(ct, gamma, arg_term))
                    .collect::<Result<Vec<_>>>()?;
                arg_term_types.iter().zip(method_type.arg_types.iter()).map(|(c, d)| {
                    match ct.is_subtype(c, d) {
                        Some(true) => Ok(()),
                        Some(false) => Err(anyhow!(
                            "Argument type `{}` is not subtype of declared type `{}` in method `{}` of class `{}`.",
                            &c.0,
                            &d.0,
                            &method_name.0,
                            &object_term_type.0
                        )),
                        None => Err(anyhow!(
                            "Class `{}` or class `{}` not in class table",
                            &c.0,
                            &d.0
                        ))
                    }
                }).collect::<Result<()>>()?;
                Ok(method_type.ret_type)
            })
        }
        // T-New
        Term::NewCall(NewCall { class_name, arg_terms }) => {
            let fields = ct.fields(class_name)
            .ok_or(anyhow!(
                "Class `{}` not in class table",
                &class_name.0
            ))?;
            let arg_term_types = arg_terms.iter().map(|arg_term| typecheck_term(ct, gamma, arg_term))
                .collect::<Result<Vec<_>>>()?;
            arg_term_types.iter().zip(fields.map(|(field_type, _)| field_type)).map(|(c, d)| {
                    match ct.is_subtype(c, d) {
                        Some(true) => Ok(()),
                        Some(false) => Err(anyhow!(
                            "Argument type `{}` is not subtype of declared type `{}` in constructor of class `{}`.",
                            &c.0,
                            &d.0,
                            &class_name.0
                        )),
                        None => Err(anyhow!(
                            "Class `{}` or class `{}` not in class table",
                            &c.0,
                            &d.0
                        ))
                    }
                }).collect::<Result<()>>()?;
            Ok(class_name.clone())
        }
        Term::Cast(Cast { to_class_name, term }) => {
            let term_type = typecheck_term(ct, gamma, term)?;
            if !ct.inner().contains_key(to_class_name) {
                bail!("Class `{}` not in class table",&to_class_name.0);
            }
            if !ct.inner().contains_key(&term_type) {
                bail!("Class `{}` not in class table",&term_type.0);
            }
            // T-UpCast
            if ct.is_subtype(&term_type, to_class_name).unwrap() {
                return Ok(to_class_name.clone())
            }
            // T-DownCast
            else if ct.is_subtype(to_class_name, &term_type).unwrap() && to_class_name != &term_type {
                return Ok(to_class_name.clone())
            }
            // T-SenselessCast
            else if !ct.is_subtype(&term_type, to_class_name).unwrap() && !ct.is_subtype(to_class_name, &term_type).unwrap() {
                println!("SENSELESS CAST WARNING: Term of type `{}` can not be cast to type `{}`", &term_type.0, &to_class_name.0);
                return Ok(to_class_name.clone())
            }
            // cast fallthrough should not happen
            else {
                bail!("No rule to cast term of type `{}` to type `{}`", &term_type.0, &to_class_name.0);
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
    let ret_term_type = typecheck_term(ct, &gamma, &method.return_term)?;
    let super_type = ct.super_type(in_class_name).ok_or(anyhow!(
        "Supertype of class `{}` not found",
        &in_class_name.0
    ))?;
    let method_type = MethodType::from_method(method);
    if !ct
        .is_correct_method_override(&method.method_name, in_class_name, &method_type)
        .ok_or(anyhow!(
            "Method `{}` in class `{}` not found",
            &method.method_name.0,
            &in_class_name.0
        ))?
    {
        bail!("Method `{}` in class `{}` does not override the method defined in the supertype correctly",
            &method.method_name.0,
            &in_class_name.0
        );
    }
    if !ct
        .is_subtype(&ret_term_type, &method_type.ret_type)
        .ok_or(anyhow!(
            "Return type of method `{}` in class `{}` not found",
            &method.method_name.0,
            &in_class_name.0
        ))?
    {
        bail!("type of returned term in method `{}` in class `{}` incorrect. Expected `{}`, actual `{}`.",
            &method.method_name.0,
            &in_class_name.0,
            &method_type.ret_type.0,
            &ret_term_type.0
        );
    }
    Ok(MethodOk(in_class_name.clone()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassOk;

pub fn typecheck_class(ct: &ClassTable, class: &ClassDefinition) -> Result<ClassOk> {
    // TODO: many checks are already done when building the class table. they might be moved here

    let super_type = ct
        .super_type(&class.name)
        .ok_or(anyhow!("Supertype of class `{}` not found", &class.name.0))?;
    let super_fields = ct.fields(&super_type).unwrap();

    // TODO: check correct super() call

    let _ = class
        .methods
        .iter()
        .map(|method| typecheck_method(ct, method, &class.name))
        .collect::<Result<Vec<_>>>()?;

    Ok(ClassOk)
}
