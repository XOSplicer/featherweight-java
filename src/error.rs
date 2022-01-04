use thiserror::Error;
use crate::ast::{ClassName, MethodName};

#[derive(Debug, Error)]
pub enum ClassTableError {

    #[error("Classes may not be named `Object`.")]
    ClassNamedObject,

    #[error("Class `{0}` is defined twice.")]
    ClassDefinedTwice(ClassName),

    #[error("The supertype `{0}` of class `{1}` is not defined.")]
    SupertypeUndefined(ClassName, ClassName),

    #[error("The supertype chain of class `{0}` contains a cycle.")]
    CyclicSupertype(ClassName),

    #[error("Contructor of class `{0}` is named `{1}`, but should be `{0}`")]
    IncorrectConstructorName(ClassName, ClassName),

    #[error("Contructor of class `{0}` does not correctly initialize all fields.")]
    IncorrectContstructorInit(ClassName),

    #[error("Class `{0}` does not have unique field names.")]
    NonUniqueFields(ClassName),

    #[error("Class `{0}` does not have unique method names.")]
    NonUniqueMethodNames(ClassName),

    #[error("Class `{0}` may not contain `this` as a field.")]
    FieldNamedThis(ClassName),

    #[error("Contructor of class `{0}` may not contain `this` as an argument.")]
    ConstructorArgumentNamedThis(ClassName),

    #[error("Method `{0}` in class `{1}` does not have unique argument names.")]
    NonUniqueMethodArgumentNames(MethodName, ClassName),

    #[error("Method `{0}` in class `{1}` may not contain `this` as an argument")]
    MethodArgumentNamedThis(MethodName, ClassName),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum TypingError {
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}
