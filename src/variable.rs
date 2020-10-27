use crate::FmiStatus;

use super::{fmi, instance, model_descr, Result};
use derive_more::Display;
use std::cmp::Ordering;
use std::rc::Rc;

// Re-exports
pub use super::model_descr::{Causality, Initial, ScalarVariableElementBase, Variability};

#[derive(Display, Debug)]
pub enum Value {
    Real(fmi::fmi2Real),
    Integer(fmi::fmi2Integer),
    Boolean(fmi::fmi2Boolean),
    String(String),
    Enumeration(fmi::fmi2Integer),
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
pub struct Var<I: instance::Common> {
    /// An owned-copy of the ScalarVariable data
    sv: model_descr::ScalarVariable, // only 120 bytes
    instance: Rc<I>,
}

impl<I: instance::Common> std::hash::Hash for Var<I> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.instance.hash(state);
        self.sv.hash(state);
    }
}

impl<I: instance::Common> Ord for Var<I> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.instance.name(), &self.sv.name).cmp(&(other.instance.name(), &other.sv.name))
    }
}

impl<I: instance::Common> PartialOrd for Var<I> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<I: instance::Common> PartialEq for Var<I> {
    fn eq(&self, other: &Self) -> bool {
        (self.instance.name(), &self.sv.name) == (other.instance.name(), &other.sv.name)
    }
}

impl<I: instance::Common> Eq for Var<I> {}

impl<I: instance::Common> Var<I> {
    /// Create a new Var from an Instance and a ScalarVariable
    pub fn from_scalar_variable(instance: &Rc<I>, sv: &model_descr::ScalarVariable) -> Self {
        Var {
            instance: instance.clone(),
            sv: sv.clone(),
        }
    }

    /// Create a new Var from an Instance given a variable name
    pub fn from_name<S: AsRef<str>>(instance: &Rc<I>, name: S) -> Result<Self> {
        let sv: &model_descr::ScalarVariable = instance
            .import()
            .descr()
            .model_variables()
            .find(|(n, _)| *n == name.as_ref())
            .map(|(_, sv)| sv)
            .ok_or_else(|| model_descr::ModelDescriptionError::VariableNotFound {
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

    pub fn scalar_variable(&self) -> &model_descr::ScalarVariable {
        &self.sv
    }

    pub fn instance(&self) -> &Rc<I> {
        &self.instance
    }

    pub fn get(&self) -> Result<Value> {
        match self.sv.elem {
            model_descr::ScalarVariableElement::Real { .. } => {
                self.instance.get_real(&self.sv).map(Value::Real)
            }
            model_descr::ScalarVariableElement::Integer { .. } => {
                self.instance.get_integer(&self.sv).map(Value::Integer)
            }
            model_descr::ScalarVariableElement::Boolean { .. } => {
                self.instance.get_boolean(&self.sv).map(Value::Boolean)
            }
            model_descr::ScalarVariableElement::String { .. } => {
                unimplemented!("String variables not supported yet.")
            }
            model_descr::ScalarVariableElement::Enumeration { .. } => {
                self.instance.get_integer(&self.sv).map(Value::Enumeration)
            }
        }
    }

    pub fn set(&self, value: &Value) -> Result<FmiStatus> {
        match (&self.sv.elem, value) {
            (model_descr::ScalarVariableElement::Real { .. }, Value::Real(x)) => {
                self.instance.set_real(&[self.sv.value_reference], &[*x])
            }
            (model_descr::ScalarVariableElement::Integer { .. }, Value::Integer(x)) => {
                self.instance.set_integer(&[self.sv.value_reference], &[*x])
            }
            (model_descr::ScalarVariableElement::Boolean { .. }, Value::Boolean(x)) => {
                self.instance.set_boolean(&[self.sv.value_reference], &[*x])
            }
            (model_descr::ScalarVariableElement::String { .. }, Value::String(_x)) => {
                unimplemented!("String variables not supported yet.")
            }
            (model_descr::ScalarVariableElement::Enumeration { .. }, Value::Enumeration(x)) => {
                self.instance.set_integer(&[self.sv.value_reference], &[*x])
            }
            _ => Err(model_descr::ModelDescriptionError::VariableTypeMismatch(
                value.into(),
                ScalarVariableElementBase::from(&self.sv.elem),
            )
            .into()),
        }
    }
}

/*
trait SetAll {
    fn set_all(self) -> Result<()>;
}

impl<'a, I> SetAll for I where I: IntoIterator<Item = &'a Value> + Clone {
    fn set_all(self) -> Result<()>  {
        let x = self.into_iter().map(|val: &Value| (val.sv.elem));

        Ok(())
    }
}

    pub fn set2<'a, T>(&self, vals: T) -> Result<()>
    where
        T: IntoIterator<Item = &'a Value>,
    {
        let q = vals.into_iter().map(|val| {
            match (&self.sv.elem, val) {
                (model_descr::ScalarVariableElement::real {..}, Value::Real(x)) => {
                    (&self.sv.value_reference, *x)
                }
                _ => Err(format_err!("Type mismatch")),
            }
        });
        //let (left, right): (Vec<_>, Vec<_>) = vec![(1,2), (3,4)].iter().cloned().unzip();

        //.collect::<(Vec<_>, Vec<_>)>();

        Ok(())
    }

*/
