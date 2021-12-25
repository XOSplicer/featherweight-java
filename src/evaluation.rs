use crate::{ast::*, class_table::ClassTable};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use std::collections::BTreeMap;
use std::iter;

pub fn eval_full(ct: &ClassTable, term: Term) -> Result<Term> {
    let mut current = term;
    while !current.is_value() {
        println!("eval_full current term: {}", &current);
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
            t => Ok(FieldAccess {
                field,
                object_term: eval_step(ct, t)?.boxed(),
            }
            .into_term()),
        },
        Term::MethodCall(MethodCall {
            mut arg_terms,
            method_name,
            object_term,
        }) => match *object_term {
            // E-Invk-New
            // NOTE: object terms be also only be values here
            Term::NewCall(nc)
                if nc.has_only_value_args() && arg_terms.iter().all(|u| u.is_value()) =>
            {
                let method_body = ct.method_body(&method_name, &nc.class_name).ok_or(anyhow!(
                    "Method `{}` in class `{}` not defined",
                    &method_name.0,
                    &nc.class_name.0
                ))?;
                let this_field = FieldName("this".into());
                let replacements = iter::once((&this_field, nc.into_term()))
                    .chain(
                        method_body
                            .args
                            .iter()
                            .zip(arg_terms.into_iter().map(|t| *t)),
                    )
                    .collect();
                Ok(substitute_many(*method_body.return_term, replacements))
            }
            // E-InvkArg
            v if v.is_value() => {
                // NOTE(unwrap): safe because of previous match arm
                let (first_non_value, _) = arg_terms
                    .iter()
                    .enumerate()
                    .find(|(_, t)| !t.is_value())
                    .unwrap();
                arg_terms[first_non_value] =
                    eval_step(ct, *arg_terms[first_non_value].clone())?.boxed();
                Ok(MethodCall {
                    arg_terms,
                    method_name,
                    object_term: v.boxed(),
                }
                .into_term())
            }
            // E-InvkRecv
            t => Ok(MethodCall {
                arg_terms,
                method_name,
                object_term: eval_step(ct, t)?.boxed(),
            }
            .into_term()),
        },
        Term::Cast(Cast {
            term,
            to_class_name,
        }) => match *term {
            // E-CastNew
            Term::NewCall(nc) if nc.has_only_value_args() => {
                if ct
                    .is_subtype(&nc.class_name, &to_class_name)
                    .ok_or(anyhow!(
                        "Class `{}` or `{}` not defined",
                        &nc.class_name.0,
                        &to_class_name.0
                    ))?
                {
                    Ok(nc.into_term())
                } else {
                    bail!(
                        "Cast failed for class `{}` to `{}`",
                        &nc.class_name.0,
                        &to_class_name.0
                    );
                }
            }
            // E-Cast
            t => Ok(Cast {
                to_class_name,
                term: eval_step(ct, t)?.boxed(),
            }
            .into_term()),
        },
        // values evaluate to themself
        Term::NewCall(nc) if nc.has_only_value_args() => Ok(nc.into_term()),
        // E-New-Arg
        Term::NewCall(NewCall {
            mut arg_terms,
            class_name,
        }) => {
            // NOTE(unwrap): safe because of previous match arm
            let (first_non_value, _) = arg_terms
                .iter()
                .enumerate()
                .find(|(_, t)| !t.is_value())
                .unwrap();
            arg_terms[first_non_value] =
                eval_step(ct, *arg_terms[first_non_value].clone())?.boxed();
            Ok(NewCall {
                arg_terms,
                class_name,
            }
            .into_term())
        }
        _ => bail!(
            "Evaluation stuck, matching not implemented, term: {}",
            &term
        ),
    }
}
