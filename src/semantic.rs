use crate::ast::*;
use anyhow::{bail, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ClassTable(BTreeMap<ClassName, ClassDefinition>);

impl ClassTable {
    pub fn try_from_ast(ast: Ast) -> Result<Self> {
        ast.class_definitions
            .into_iter()
            .map(|c: ClassDefinition| {
                if c.name.0 == "Object" {
                    bail!("Classes may not be named `Object`")
                }
                Ok((c.name.clone(), c))
            })
            .collect::<Result<BTreeMap<_, _>>>()
            .map(ClassTable)
    }

    pub fn inner(&self) -> &BTreeMap<ClassName, ClassDefinition> {
        &self.0
    }

    pub fn super_type(&self, class_name: &ClassName) -> Option<&ClassName> {
        self.inner().get(class_name).map(|class| &class.super_type)
    }

    pub fn fields(
        &self,
        class_name: &ClassName,
    ) -> Option<Box<dyn Iterator<Item = &ArgPair> + '_>> {
        if class_name.0 == "Object" {
            return Some(Box::new(std::iter::empty()));
        }
        self.inner().get(class_name).map(|class| {
            Box::new(
                class
                    .fields
                    .iter()
                    .chain(self.fields(self.super_type(class_name).unwrap()).unwrap()),
            ) as Box<_>
        })
    }

    pub fn method_type(
        &self,
        method_name: &MethodName,
        class_name: &ClassName,
    ) -> Option<MethodType> {
        self.inner().get(class_name).and_then(|class| {
            match class
                .methods
                .iter()
                .find(|method| &method.method_name == method_name)
            {
                Some(method) => Some(MethodType::from_method(method)),
                None => self.method_type(method_name, self.super_type(class_name).unwrap()),
            }
        })
    }
}

#[derive(Debug, Clone)]
pub struct MethodType {
    pub arg_types: Vec<ClassName>,
    pub ret_type: ClassName,
}

impl MethodType {
    pub fn from_method(method: &MethodDefinition) -> Self {
        MethodType {
            arg_types: method
                .args
                .iter()
                .map(|(class_name, _)| class_name.clone())
                .collect(),
            ret_type: method.return_type.clone(),
        }
    }
}
