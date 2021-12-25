use crate::{ast::*, class_table::ClassTable};
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
use std::iter;

pub fn eval_full(ct: &ClassTable, term: Term) -> Result<Term> {
    let mut current = term;
    while !current.is_value() {
        current = eval_step(ct, current)?;
    }
    Ok(current)
}

impl Term {
    fn is_value(&self) -> bool {
        match self {
            Term::NewCall(nc) => nc.has_only_value_args(),
            _ => false,
        }
    }
}

impl NewCall {
    fn has_only_value_args(&self) -> bool {
        self.arg_terms.iter().all(|term| term.is_value())
    }
}

fn substitute(in_term: Term, to_replace: &FieldName, with_term: Term) -> Term {
    match in_term {
        Term::Variable(v) => {
            if &v == to_replace {
                with_term
            } else {
                Term::Variable(v)
            }
        }
        Term::Cast(Cast {
            term,
            to_class_name,
        }) => Term::Cast(Cast {
            term: substitute(*term, to_replace, with_term).boxed(),
            to_class_name,
        }),
        Term::FieldAccess(FieldAccess { field, object_term }) => Term::FieldAccess(FieldAccess {
            field,
            object_term: substitute(*object_term, to_replace, with_term).boxed(),
        }),
        Term::MethodCall(MethodCall {
            arg_terms,
            method_name,
            object_term,
        }) => Term::MethodCall(MethodCall {
            arg_terms: arg_terms
                .into_iter()
                .map(|term| substitute(*term, to_replace, with_term.clone()).boxed())
                .collect(),
            method_name,
            object_term: substitute(*object_term, to_replace, with_term.clone()).boxed(),
        }),
        Term::NewCall(NewCall {
            arg_terms,
            class_name,
        }) => Term::NewCall(NewCall {
            arg_terms: arg_terms
                .into_iter()
                .map(|term| substitute(*term, to_replace, with_term.clone()).boxed())
                .collect(),
            class_name,
        }),
    }
}

fn substitute_many(in_term: Term, replacements: BTreeMap<&FieldName, Term>) -> Term {
    let mut current = in_term;
    for (to_replace, with_term) in replacements {
        current = substitute(current, to_replace, with_term);
    }
    current
}

pub fn eval_step(ct: &ClassTable, term: Term) -> Result<Term> {
    match term {
        Term::FieldAccess(FieldAccess { field, object_term }) => match *object_term {
            // E-ProjNew
            Term::NewCall(nc) if nc.has_only_value_args() => {
                // check that field in class
                let (i, _) = ct
                    .fields(&nc.class_name)
                    .ok_or(anyhow!("Class `{}` is not defined", &nc.class_name.0))?
                    .enumerate()
                    .find(|(_, (_, field_name))| field_name == &field)
                    .ok_or(anyhow!(
                        "Field `{}` not defined in class `{}`",
                        &field.0,
                        &nc.class_name.0
                    ))?;
                Ok(*nc
                    .arg_terms
                    .get(i)
                    .ok_or(anyhow!("Could not get contructor arg"))?
                    .clone())
            }
            // E-Field
            _ => todo!(),
        },
        Term::MethodCall(MethodCall {
            arg_terms,
            method_name,
            object_term,
        }) => match *object_term {
            // E-Invk-New
            Term::NewCall(nc) if nc.has_only_value_args() => {
                let method_body = ct.method_body(&method_name, &nc.class_name).ok_or(anyhow!(
                    "Method `{}` in class `{}` not defined",
                    &method_name.0,
                    &nc.class_name.0
                ))?;
                let this_field = FieldName("this".into());
                let replacements =
                    iter::once((&this_field, Term::NewCall(nc.clone())))
                        .chain(
                            method_body
                                .args
                                .iter()
                                .zip(arg_terms.into_iter().map(|t| *t)),
                        )
                        .collect();
                Ok(substitute_many(*method_body.return_term, replacements))
            }
            _ => todo!(),
        },
        _ => todo!(),
    }
}