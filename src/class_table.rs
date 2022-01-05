use crate::{ast::*, error::ClassTableError};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub struct ClassTable(BTreeMap<ClassName, ClassDefinition>);

impl ClassTable {
    pub fn try_from_ast(ast: Ast) -> Result<Self, ClassTableError> {
        let mut map = BTreeMap::new();

        // transform ast to class table
        // and check that no class is defined more than once
        for class in ast.class_definitions.into_iter() {
            if class.name.is_object() {
                Err(ClassTableError::ClassNamedObject)?;
            }
            if let Some(c) = map.insert(class.name.clone(), class) {
                Err(ClassTableError::ClassDefinedTwice(c.name))?;
            }
        }

        let ct = ClassTable(map);

        // check that each supertype is defined
        for class in ct.inner().values() {
            let supertype = &class.super_type;
            if !(ct.inner().contains_key(supertype) || supertype.is_object()) {
                Err(ClassTableError::SupertypeUndefined(
                    supertype.clone(),
                    class.name.clone(),
                ))?;
            }
        }

        // TODO: maybe there checks should be part of typechecking

        // - check that the class table is acyclic
        // (this must be checked after the previous check)
        // - check that no field is defined twice in a class
        // - check that each class has a contructor with the correct name
        // - check that a class assigns all fields in the ctor
        // - check that no method is defined twice in a class
        // - check that methods have unique argument names
        // - check that class fields are not named `this`
        // - check that method/ctor args are not named `this`
        for class in ct.inner().values() {
            if ct.super_type_chain(&class.name).unwrap().is_cyclic() {
                Err(ClassTableError::CyclicSupertype(class.name.clone()))?;
            }
            if !class.has_correct_ctor_name() {
                Err(ClassTableError::IncorrectConstructorName(
                    class.name.clone(),
                    class.constructor.name.clone(),
                ))?;
            }
            if !class.has_correct_ctor_init() {
                Err(ClassTableError::IncorrectContstructorInit(
                    class.name.clone(),
                ))?;
            }
            if !class.has_unique_field_names() {
                Err(ClassTableError::NonUniqueFields(class.name.clone()))?;
            }
            if !class.has_unique_method_names() {
                Err(ClassTableError::NonUniqueMethodNames(class.name.clone()))?;
            }
            if !class.has_only_valid_field_names() {
                Err(ClassTableError::FieldNamedThis(class.name.clone()))?;
            }
            if !class.constructor.has_only_valid_argument_names() {
                Err(ClassTableError::ConstructorArgumentNamedThis(
                    class.name.clone(),
                ))?;
            }

            for method in class.methods.iter() {
                if !method.has_unique_argument_names() {
                    Err(ClassTableError::NonUniqueMethodArgumentNames(
                        method.method_name.clone(),
                        class.name.clone(),
                    ))?;
                }
                if !method.has_only_valid_argument_names() {
                    Err(ClassTableError::MethodArgumentNamedThis(
                        method.method_name.clone(),
                        class.name.clone(),
                    ))?;
                }
            }
        }

        // FIXME: check of correct override is not working as intended
        // return type is ignored

        Ok(ct)
    }

    pub fn inner(&self) -> &BTreeMap<ClassName, ClassDefinition> {
        &self.0
    }

    pub fn super_type(&self, class_name: &ClassName) -> Option<&ClassName> {
        self.inner().get(class_name).map(|class| &class.super_type)
    }

    pub fn super_type_chain<'a>(&'a self, class_name: &'a ClassName) -> Option<SuperTypeChain<'a>> {
        if class_name.is_object() {
            return Some(SuperTypeChain {
                ct: &self,
                last: class_name,
            });
        }
        self.inner().get(class_name).map(|class| SuperTypeChain {
            ct: &self,
            last: &class.name,
        })
    }

    pub fn is_subtype(&self, lhs: &ClassName, rhs: &ClassName) -> Option<bool> {
        if lhs.is_object() && rhs.is_object() {
            return Some(true);
        }
        if lhs == rhs && self.contains_class(lhs) {
            return Some(true);
        }
        if rhs.is_object() && self.contains_class(lhs) {
            return Some(true);
        }
        // dbg!(lhs, rhs);
        self.super_type_chain(lhs)
            .map(|mut super_types| super_types.any(|s| s == rhs))
    }

    pub fn contains_class(&self, class_name: &ClassName) -> bool {
        self.inner().contains_key(class_name) || class_name.is_object()
    }

    pub fn subtypes<'a>(
        &'a self,
        class_name: &'a ClassName,
    ) -> Option<impl Iterator<Item = &'a ClassName>> {
        if !(self.contains_class(class_name)) {
            return None;
        }
        Some(
            self.inner()
                .keys()
                .filter(|&t| self.is_subtype(t, class_name).unwrap()),
        )
    }

    pub fn direct_subtypes<'a>(
        &'a self,
        class_name: &'a ClassName,
    ) -> Option<impl Iterator<Item = &'a ClassName>> {
        if !(self.contains_class(class_name)) {
            return None;
        }
        Some(
            self.inner()
                .values()
                .filter(move |&c| &c.super_type == class_name && &c.name != class_name)
                .map(|c| &c.name),
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

    pub fn method_body(
        &self,
        method_name: &MethodName,
        class_name: &ClassName,
    ) -> Option<MethodBody> {
        self.inner().get(class_name).and_then(|class| {
            match class
                .methods
                .iter()
                .find(|method| &method.method_name == method_name)
            {
                Some(method) => Some(MethodBody::from_method(method)),
                None => self.method_body(method_name, self.super_type(class_name).unwrap()),
            }
        })
    }

    pub fn is_correct_method_override(
        &self,
        method_name: &MethodName,
        class_name: &ClassName,
        method_type: &MethodType,
    ) -> Option<bool> {
        let expected_method_type = self.method_type(method_name, class_name)?;
        Some(
            expected_method_type.arg_types == method_type.arg_types
                && expected_method_type.ret_type == method_type.ret_type,
        )
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

#[derive(Debug, Clone)]
pub struct MethodBody {
    pub args: Vec<FieldName>,
    pub return_term: Box<Term>,
}

impl MethodBody {
    pub fn from_method(method: &MethodDefinition) -> Self {
        MethodBody {
            args: method
                .args
                .iter()
                .map(|(_, field_name)| field_name.clone())
                .collect(),
            return_term: method.return_term.clone(),
        }
    }
}

impl ClassName {
    pub fn is_object(&self) -> bool {
        self.0 == "Object"
    }
}

impl FieldName {
    pub fn is_this(&self) -> bool {
        self.0 == "this"
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

impl ClassDefinition {
    fn has_correct_ctor_name(&self) -> bool {
        self.name == self.constructor.name
    }
    fn has_correct_ctor_init(&self) -> bool {
        // check that left and right side of assignments have the same field name
        self.constructor
            .assignments
            .iter()
            .all(|(lhs, rhs)| lhs == rhs);

        let mut init_fields = BTreeSet::new();
        // check no double init
        for field_name in self
            .constructor
            .assignments
            .iter()
            .map(|(field_name, _)| field_name)
        {
            if !init_fields.insert(field_name) {
                return false;
            }
        }
        let init_fields = init_fields;
        // check for doubles in below method
        let class_fields: BTreeSet<_> = self
            .fields
            .iter()
            .map(|(_, field_name)| field_name)
            .collect();
        let all_inited = init_fields.difference(&class_fields).count() == 0;

        all_inited
    }
    fn has_unique_field_names(&self) -> bool {
        let mut seen = BTreeSet::new();
        for field_name in self.fields.iter().map(|(_, field_name)| field_name) {
            if !seen.insert(field_name) {
                return false;
            }
        }
        true
    }
    fn has_unique_method_names(&self) -> bool {
        let mut seen = BTreeSet::new();
        for method_name in self.methods.iter().map(|method| &method.method_name) {
            if !seen.insert(method_name) {
                return false;
            }
        }
        true
    }

    fn has_only_valid_field_names(&self) -> bool {
        self.fields
            .iter()
            .map(|(_, arg_name)| arg_name)
            .all(|arg_name| !arg_name.is_this())
    }
}

impl MethodDefinition {
    fn has_unique_argument_names(&self) -> bool {
        let mut seen = BTreeSet::new();
        for arg_name in self.args.iter().map(|(_, arg_name)| arg_name) {
            if !seen.insert(arg_name) {
                return false;
            }
        }
        true
    }
    fn has_only_valid_argument_names(&self) -> bool {
        self.args
            .iter()
            .map(|(_, arg_name)| arg_name)
            .all(|arg_name| !arg_name.is_this())
    }
}

impl Constructor {
    fn has_only_valid_argument_names(&self) -> bool {
        self.args
            .iter()
            .map(|(_, arg_name)| arg_name)
            .all(|arg_name| !arg_name.is_this())
    }
}
