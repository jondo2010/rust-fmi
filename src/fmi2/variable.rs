use crate::FmiStatus;

use super::FmiResult;
use derive_more::Display;
use std::cmp::Ordering;

// Re-exports
pub use super::fmi2::meta::{Causality, Initial, ScalarVariableElementBase, Variability};

#[derive(Display, Debug)]
pub enum Value {
    Real(fmi2::fmi2Real),
    Integer(fmi2::fmi2Integer),
    Boolean(fmi2::fmi2Boolean),
    String(String),
    Enumeration(fmi2::fmi2Integer),
}

impl From<&Value> for ScalarVariableElementBase {
    fn from(value: &Value) -> Self {
        match value {
            Value::Real(_) => Self::Real,
            Value::Integer(_) => Self::Integer,
            Value::Boolean(_) => Self::Boolean,
            Value::String(_) => Self::String,
            Value::Enumeration(_) => Self::Enumeration,
        }
    }
}

/// Var wraps access to an underlying ScalarVariable on an Instance
#[derive(Display, Debug)]
#[display(fmt = "Var {}.{}", "self.instance.name()", "self.name()")]
pub struct Var<'inst, I: instance::Common> {
    /// An owned-copy of the ScalarVariable data
    sv: meta::ScalarVariable, // only 120 bytes
    instance: &'inst I,
}

impl<'inst, I: instance::Common> std::hash::Hash for Var<'inst, I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.instance.hash(state);
        self.sv.hash(state);
    }
}

impl<'inst, I: instance::Common> Ord for Var<'inst, I> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.instance.name(), &self.sv.name).cmp(&(other.instance.name(), &other.sv.name))
    }
}

impl<'inst, I: instance::Common> PartialOrd for Var<'inst, I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'inst, I: instance::Common> PartialEq for Var<'inst, I> {
    fn eq(&self, other: &Self) -> bool {
        (self.instance.name(), &self.sv.name) == (other.instance.name(), &other.sv.name)
    }
}

impl<'inst, I: instance::Common> Eq for Var<'inst, I> {}

impl<'inst, I: instance::Common> Var<'inst, I> {
    /// Create a new Var from an Instance and a ScalarVariable
    pub fn from_scalar_variable(instance: &'inst I, sv: &meta::ScalarVariable) -> Self {
        Var {
            instance,
            sv: sv.clone(),
        }
    }

    /// Create a new Var from an Instance given a variable name
    #[cfg(feature = "disable")]
    pub fn from_name<S: AsRef<str>>(instance: &'inst I, name: S) -> Result<Self> {
        let sv: &meta::ScalarVariable = instance
            .import()
            .descr()
            .get_model_variables()
            .find(|(_vr, sv)| sv.name == name.as_ref())
            .map(|(_vr, sv)| sv)
            .ok_or_else(|| meta::ModelDescriptionError::VariableNotFound {
                model: instance.import().descr().model_name().to_owned(),
                name: name.as_ref().into(),
            })?;

        let instance = instance.clone();

        Ok(Var {
            instance,
            sv: sv.clone(),
        })
    }

    pub fn name(&self) -> &str {
        &self.sv.name
    }

    pub fn scalar_variable(&self) -> &meta::ScalarVariable {
        &self.sv
    }

    pub fn instance(&self) -> &I {
        self.instance
    }

    pub fn get(&self) -> Result<Value> {
        match self.sv.elem {
            meta::ScalarVariableElement::Real { .. } => {
                self.instance.get_real(&self.sv).map(Value::Real)
            }
            meta::ScalarVariableElement::Integer { .. } => {
                self.instance.get_integer(&self.sv).map(Value::Integer)
            }
            meta::ScalarVariableElement::Boolean { .. } => {
                self.instance.get_boolean(&self.sv).map(Value::Boolean)
            }
            meta::ScalarVariableElement::String { .. } => {
                unimplemented!("String variables not supported yet.")
            }
            meta::ScalarVariableElement::Enumeration { .. } => {
                self.instance.get_integer(&self.sv).map(Value::Enumeration)
            }
        }
    }

    pub fn set(&self, value: &Value) -> Result<FmiStatus> {
        match (&self.sv.elem, value) {
            (meta::ScalarVariableElement::Real { .. }, Value::Real(x)) => {
                self.instance.set_real(&[self.sv.value_reference], &[*x])
            }
            (meta::ScalarVariableElement::Integer { .. }, Value::Integer(x)) => {
                self.instance.set_integer(&[self.sv.value_reference], &[*x])
            }
            (meta::ScalarVariableElement::Boolean { .. }, Value::Boolean(x)) => self
                .instance
                .set_boolean(&[self.sv.value_reference.0], &[*x]),
            (meta::ScalarVariableElement::String { .. }, Value::String(_x)) => {
                unimplemented!("String variables not supported yet.")
            }
            (meta::ScalarVariableElement::Enumeration { .. }, Value::Enumeration(x)) => {
                self.instance.set_integer(&[self.sv.value_reference], &[*x])
            }
            _ => Err(meta::ModelDescriptionError::VariableTypeMismatch(
                value.into(),
                ScalarVariableElementBase::from(&self.sv.elem),
            )
            .into()),
        }
    }
}

// trait SetAll {
// fn set_all(self) -> Result<()>;
// }
//
// impl<'a, I> SetAll for I where I: IntoIterator<Item = &'a Value> + Clone {
// fn set_all(self) -> Result<()>  {
// let x = self.into_iter().map(|val: &Value| (val.sv.elem));
//
// Ok(())
// }
// }
//
// pub fn set2<'a, T>(&self, vals: T) -> Result<()>
// where
// T: IntoIterator<Item = &'a Value>,
// {
// let q = vals.into_iter().map(|val| {
// match (&self.sv.elem, val) {
// (meta::ScalarVariableElement::real {..}, Value::Real(x)) => {
// (&self.sv.value_reference, *x)
// }
// _ => Err(format_err!("Type mismatch")),
// }
// });
// let (left, right): (Vec<_>, Vec<_>) = vec![(1,2), (3,4)].iter().cloned().unzip();
//
// .collect::<(Vec<_>, Vec<_>)>();
//
// Ok(())
// }
//
