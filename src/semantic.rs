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
}
