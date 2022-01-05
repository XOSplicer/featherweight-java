use std::{collections::BTreeMap, fmt::Display};

use crate::ast::ClassName;
use crate::class_table::ClassTable;

#[derive(Debug, Clone)]
pub struct ClassTree(BTreeMap<ClassName, Box<ClassTree>>);

impl ClassTree {
    pub fn new(ct: &ClassTable) -> Self {
        Self::new_for(ct, &ClassName::object())
    }
    fn new_for(ct: &ClassTable, c: &ClassName) -> Self {
        let mut map = BTreeMap::new();
        let subtypes = ct
            .direct_subtypes(c)
            .unwrap()
            .map(|s| Self::new_for(ct, s).0)
            .fold(BTreeMap::default(), |mut acc, m| {
                acc.extend(m.into_iter());
                acc
            });
        map.insert(c.clone(), Box::new(ClassTree(subtypes)));
        ClassTree(map)
    }
}

impl Display for ClassTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl ClassName {
    pub fn object() -> Self {
        ClassName("Object".into())
    }
}
