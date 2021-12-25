use crate::ast::*;
use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct ClassTable(BTreeMap<ClassName, ClassDefinition>);

impl ClassTable {
    pub fn try_from_ast(ast: Ast) -> Result<Self> {
        let mut map = BTreeMap::new();

        // transform ast to class table
        // and check that no class is defined more than once
        for class in ast.class_definitions.into_iter() {
            if class.name.is_object() {
                bail!("Classes may not be named `Object`.");
            }
            if let Some(c) = map.insert(class.name.clone(), class) {
                bail!("Class `{}` is defined twice.", &c.name.0);
            }
        }

        let ct = ClassTable(map);

        // check that each supertype is defined
        for class in ct.inner().values() {
            let super_type = &class.super_type;
            if !(ct.inner().contains_key(super_type) || super_type.is_object()) {
                bail!(
                    "The supertype `{}` of class `{}` is not defined.",
                    &super_type.0,
                    &class.name.0
                );
            }
        }

        // check that the class table is acyclic
        // must be checked after the previous check
        for class in ct.inner().values() {
            if ct.super_type_chain(&class.name).unwrap().is_cyclic() {
                bail!(
                    "The supertype chain of class `{}` contains a cycle",
                    &class.name.0
                )
            }
        }

        Ok(ct)
    }

    pub fn inner(&self) -> &BTreeMap<ClassName, ClassDefinition> {
        &self.0
    }

    pub fn super_type(&self, class_name: &ClassName) -> Option<&ClassName> {
        self.inner().get(class_name).map(|class| &class.super_type)
    }

    pub fn super_type_chain(&self, class_name: &ClassName) -> Option<SuperTypeChain<'_>> {
        self.inner().get(class_name).map(|class| SuperTypeChain {
            ct: &self,
            last: &class.name,
        })
    }

    pub fn is_subtype(&self, lhs: &ClassName, rhs: &ClassName) -> Option<bool> {
        self.super_type_chain(lhs)
            .map(|mut super_types| super_types.any(|s| s == rhs))
    }

    pub fn subtypes<'a>(
        &'a self,
        class_name: &'a ClassName,
    ) -> Option<impl Iterator<Item = &'a ClassName>> {
        if !(self.inner().contains_key(class_name) || class_name.is_object()) {
            return None;
        }
        Some(
            self.inner()
                .keys()
                .filter(|&t| self.is_subtype(t, class_name).unwrap()),
        )
    }

    pub fn fields(
        &self,
        class_name: &ClassName,
    ) -> Option<Box<dyn Iterator<Item = &ArgPair> + '_>> {
        if class_name.is_object() {
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

impl ClassName {
    fn is_object(&self) -> bool {
        self.0 == "Object"
    }
}

pub struct SuperTypeChain<'a> {
    ct: &'a ClassTable,
    last: &'a ClassName,
}

impl<'a> SuperTypeChain<'a> {
    fn is_cyclic(self) -> bool {
        let mut seen = BTreeSet::new();
        for class_name in self {
            if !seen.insert(class_name) {
                return true;
            }
        }
        false
    }
}

impl<'a> Iterator for SuperTypeChain<'a> {
    type Item = &'a ClassName;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last.is_object() {
            return None;
        }
        let next = self.ct.super_type(self.last).unwrap();
        self.last = next;
        Some(next)
    }
}
