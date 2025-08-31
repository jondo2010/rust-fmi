//! FMI 3.0 instance interface

use crate::{
    CS, InterfaceType, ME, SE,
    fmi3::{
        Fmi3Error, Fmi3Res,
        traits::{Common, GetSet},
    },
    traits::{FmiImport, FmiInstance, InstanceTag},
};

use super::{Fmi3Status, binding, import::Fmi3Import, schema};

mod co_simulation;
mod common;
mod model_exchange;
mod scheduled_execution;

use itertools::Itertools;

pub type InstanceME<'a> = Instance<'a, ME>;
pub type InstanceCS<'a> = Instance<'a, CS>;
pub type InstanceSE<'a> = Instance<'a, SE>;

/// An imported FMI 3.0 instance
pub struct Instance<'a, Tag> {
    /// Raw FMI 3.0 bindings
    binding: binding::Fmi3Binding,
    /// Pointer to the raw FMI 3.0 instance
    ptr: binding::fmi3Instance,
    /// Model description
    model_description: &'a schema::Fmi3ModelDescription,
    /// Instance name
    name: String,
    _tag: std::marker::PhantomData<&'a Tag>,
}

impl<'a, Tag> Drop for Instance<'a, Tag> {
    fn drop(&mut self) {
        unsafe {
            log::trace!("Freeing instance {:?}", self.ptr);
            self.binding.fmi3FreeInstance(self.ptr);
        }
    }
}

impl<'a, Tag> Instance<'a, Tag>
where
    Self: Common,
{
    /// Get the sum of the product of the dimensions of the variables with the given value references.
    pub fn get_variable_dimensions(&mut self, var_refs: &[u32]) -> usize {
        let var_dims = var_refs.iter().map(|vr| {
            self.model_description
                .model_variables
                .iter_floating()
                .find(|fv| fv.value_reference() == *vr)
                .map(|fv| fv.dimensions().iter())
                .expect("Variable not found")
        });

        var_dims
            .map(|dims| {
                dims.map(
                    |schema::Dimension {
                         start,
                         value_reference,
                     }| {
                        match (start, value_reference) {
                            // If the dimension has a start and no value reference, it is a constant
                            (&Some(start), None) => start as usize,
                            // If the dimension has a ValueRef, then it could be dynamically set during configuration mode
                            (None, &Some(vr)) => {
                                let mut dim_val = [0];
                                self.get_uint64(&[vr.into()], &mut dim_val)
                                    .ok()
                                    .expect("Error getting dimension");
                                dim_val[0] as usize
                            }
                            _ => panic!("Invalid Dimension"),
                        }
                    },
                )
                .product::<usize>()
            })
            .sum()
    }
}

impl<'a, Tag: InstanceTag> FmiInstance for Instance<'a, Tag> {
    type ModelDescription = schema::Fmi3ModelDescription;
    type ValueRef = <Fmi3Import as FmiImport>::ValueRef;
    type Status = Fmi3Status;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        Common::get_version(self)
    }

    fn interface_type(&self) -> InterfaceType {
        Tag::TYPE
    }

    fn model_description(&self) -> &Self::ModelDescription {
        self.model_description
    }

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi3Res, Fmi3Error> {
        Common::set_debug_logging(self, logging_on, categories)
    }

    fn get_number_of_continuous_state_values(&mut self) -> usize {
        let md = self.model_description();
        let cts_vars = md
            .model_structure
            .continuous_state_derivative
            .iter()
            .map(|csd| csd.value_reference)
            .collect_vec();
        self.get_variable_dimensions(&cts_vars)
    }

    fn get_number_of_event_indicators(&mut self) -> usize {
        let md = self.model_description();
        let event_vars = md
            .model_structure
            .event_indicator
            .iter()
            .map(|ei| ei.value_reference)
            .collect_vec();
        self.get_variable_dimensions(&event_vars)
    }

    fn enter_initialization_mode(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        Common::enter_initialization_mode(self, tolerance, start_time, stop_time)
    }

    fn exit_initialization_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Common::exit_initialization_mode(self)
    }

    fn terminate(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Common::terminate(self)
    }

    fn reset(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Common::reset(self)
    }
}

pub struct Fmu3State<'a, Tag> {
    instance: Instance<'a, Tag>,
    /// Pointer to the raw FMI 3.0 state
    state: binding::fmi3FMUState,
}

impl<'a, Tag> Drop for Fmu3State<'a, Tag> {
    fn drop(&mut self) {
        unsafe {
            log::trace!("Freeing state {:?}", self.state);
            self.instance
                .binding
                .fmi3FreeFMUState(self.instance.ptr, &mut self.state);
        }
    }
}
