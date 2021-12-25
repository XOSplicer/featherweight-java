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
    todo!()
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
