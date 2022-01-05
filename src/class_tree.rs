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
        let printer = Printer {
            tree: self,
            indent: 0,
        };
        Display::fmt(&printer, f)
    }
}

struct Printer<'a> {
    indent: usize,
    tree: &'a ClassTree,
}

impl<'a> Display for Printer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in self.tree.0.iter() {
            let sub_printer = Printer {
                indent: self.indent + 2,
                tree: &*v,
            };
            for i in 0..self.indent {
                write!(f, " ")?;
            }
            writeln!(f, "└{}", k)?;
            Display::fmt(&sub_printer, f)?;
        }
        Ok(())
    }
}

impl ClassName {
    pub fn object() -> Self {
        ClassName("Object".into())
    }
}
