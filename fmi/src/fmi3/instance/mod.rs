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

pub type InstanceME = Instance<ME>;
pub type InstanceCS = Instance<CS>;
pub type InstanceSE = Instance<SE>;

/// An imported FMI 3.0 instance
pub struct Instance<Tag> {
    /// Raw FMI 3.0 bindings
    binding: binding::Fmi3Binding,
    /// Pointer to the raw FMI 3.0 instance
    ptr: binding::fmi3Instance,
    /// Instance name
    name: String,
    _tag: std::marker::PhantomData<Tag>,
}

impl<Tag> Drop for Instance<Tag> {
    fn drop(&mut self) {
        unsafe {
            log::trace!("Freeing instance {:?}", self.ptr);
            self.binding.fmi3FreeInstance(self.ptr);
        }
    }
}

impl<Tag> Instance<Tag>
where
    Self: Common,
{
    /// Get the sum of the product of the dimensions of the variables with the given value references.
    ///
    /// # Arguments
    /// * `model_description` - The model description to look up variable information
    /// * `var_refs` - Value references of the variables to get dimensions for
    pub fn get_variable_dimensions(
        &mut self,
        model_description: &schema::Fmi3ModelDescription,
        var_refs: &[u32],
    ) -> usize {
        let var_dims = var_refs.iter().map(|vr| {
            model_description
                .model_variables
                .iter_floating()
                .find(|fv| fv.value_reference() == *vr)
                .map(|fv| fv.dimensions().iter())
                .expect("Variable not found")
        });

        var_dims
            .map(|dims| {
                dims.map(|dim| match dim {
                    schema::Dimension::Fixed(start) => *start as usize,
                    schema::Dimension::Variable(vr) => {
                        let mut dim_val = [0];
                        self.get_uint64(&[(*vr).into()], &mut dim_val)
                            .expect("Error getting dimension");
                        dim_val[0] as usize
                    }
                })
                .product::<usize>()
            })
            .sum()
    }
}

impl<Tag: InstanceTag> FmiInstance for Instance<Tag> {
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

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi3Res, Fmi3Error> {
        Common::set_debug_logging(self, logging_on, categories)
    }

    fn get_number_of_continuous_state_values(&mut self) -> usize {
        // For FMI3, we return 0 as a default. 
        // ME instances should call get_number_of_continuous_states() directly via the ModelExchange trait.
        0
    }

    fn get_number_of_event_indicators(&mut self) -> usize {
        // For FMI3, we return 0 as a default.
        // ME instances should call get_number_of_event_indicators() directly via the ModelExchange trait.
        0
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

pub struct Fmu3State<Tag> {
    instance: Instance<Tag>,
    /// Pointer to the raw FMI 3.0 state
    state: binding::fmi3FMUState,
}

impl<Tag> Drop for Fmu3State<Tag> {
    fn drop(&mut self) {
        unsafe {
            log::trace!("Freeing state {:?}", self.state);
            self.instance
                .binding
                .fmi3FreeFMUState(self.instance.ptr, &mut self.state);
        }
    }
}
