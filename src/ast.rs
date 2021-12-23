#[derive(Debug, Clone)]
pub struct Ast {
    pub class_definitions: Vec<ClassDefinition>,
}

pub type ArgPair = (ClassName, FieldName);

#[derive(Debug, Clone)]
pub struct ClassDefinition {
    pub name: ClassName,
    pub super_type: ClassName,
    pub fields: Vec<ArgPair>,
    pub constructor: Constructor,
    pub methods: Vec<MethodDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClassName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldName(pub String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MethodName(pub String);

#[derive(Debug, Clone)]
pub struct Constructor {
    pub name: ClassName,
    pub args: Vec<ArgPair>,
    pub super_call: Vec<FieldName>,
    pub assignments: Vec<(FieldName, FieldName)>,
}

#[derive(Debug, Clone)]
pub struct MethodDefinition {
    pub return_type: ClassName,
    pub method_name: MethodName,
    pub args: Vec<ArgPair>,
    pub return_term: Box<Term>,
}

#[derive(Debug, Clone)]
pub enum Term {
    Variable(FieldName),
    FieldAccess(FieldAccess),
    MethodCall(MethodCall),
    NewCall(NewCall),
    Cast(Cast),
}

impl Term {
    pub fn boxed(self) -> Box<Self> {
        Box::new(self)
    }
    pub fn from_variable_str(name: &str) -> Term {
        Term::Variable(FieldName(name.into()))
    }
}

#[derive(Debug, Clone)]
pub struct FieldAccess {
    pub object_term: Box<Term>,
    pub field: FieldName,
}

impl FieldAccess {
    pub fn into_term(self) -> Term {
        Term::FieldAccess(self)
    }
}

#[derive(Debug, Clone)]
pub struct MethodCall {
    pub object_term: Box<Term>,
    pub method_name: MethodName,
    pub arg_terms: Vec<Box<Term>>,
}

impl MethodCall {
    pub fn into_term(self) -> Term {
        Term::MethodCall(self)
    }
}

#[derive(Debug, Clone)]
pub struct NewCall {
    pub class_name: ClassName,
    pub arg_terms: Vec<Box<Term>>,
}

impl NewCall {
    pub fn into_term(self) -> Term {
        Term::NewCall(self)
    }
}

#[derive(Debug, Clone)]
pub struct Cast {
    pub to_class_name: ClassName,
    pub term: Box<Term>,
}

impl Cast {
    pub fn into_term(self) -> Term {
        Term::Cast(self)
    }
}

// TODO: value trait?