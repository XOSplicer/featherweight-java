use std::collections::BTreeMap;

use crate::{ast::*, class_table::ClassTable};
use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Gamma {
    inner: BTreeMap<FieldName, ClassName>,
}


pub fn typecheck_term(ct: &ClassTable, gamma: &Gamma, term: &Term) -> Result<ClassName> {
    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodOk(/* in */ pub ClassName);

pub fn typecheck_method(
    ct: &ClassTable,
    gamma: &Gamma,
    method: &MethodDefinition,
    in_class_name: &ClassName,
) -> Result<MethodOk> {
    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassOk;

pub fn typecheck_class(ct: &ClassTable, gamma: &Gamma, class: &ClassDefinition) {
    todo!()
}
