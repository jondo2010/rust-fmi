use arrow::array::RecordBatch;
use std::path::Path;

use fmi::{
    EventFlags,
    traits::{FmiEventHandler, FmiImport, FmiInstance},
};

pub use io::{InputState, RecorderState};

use crate::{Error, options};

use self::{
    interpolation::Linear,
    params::SimParams,
    traits::{FmiSim, InstRecordValues, InstSetValues, SimDefaultInitialize, SimHandleEvents},
};

#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
mod interpolation;
mod io;
mod me;
pub mod params;
pub mod solver;
pub mod traits;
pub mod util;

pub struct SimState<Inst>
where
    Inst: FmiInstance,
{
    sim_params: SimParams,
    input_state: InputState<Inst>,
    recorder_state: RecorderState<Inst>,
    inst: Inst,
    event_flags: EventFlags,
}

impl<Inst: FmiInstance> SimState<Inst> {
    /// Get the next event time, or `f64::INFINITY` if none is scheduled.
    pub fn next_event_time(&self) -> f64 {
        self.event_flags.next_event_time.unwrap_or(f64::INFINITY)
    }
}

pub trait SimStateTrait<'a, Inst: FmiInstance, Import: FmiImport> {
    fn new(
        import: &'a Import,
        sim_params: SimParams,
        input_state: InputState<Inst>,
        output_state: RecorderState<Inst>,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

impl<Inst> SimHandleEvents for SimState<Inst>
where
    Inst: FmiEventHandler + InstSetValues + InstRecordValues,
{
    fn handle_events(&mut self, time: f64, input_event: bool) -> Result<(bool, bool), Error> {
        self.inst.record_outputs(time, &mut self.recorder_state)?;
        self.inst.enter_event_mode().map_err(Into::into)?;
        if input_event {
            self.input_state
                .apply_input::<Linear>(time, &mut self.inst, true, true, true)?;
        }
        let mut reset_solver = false;

        self.event_flags.discrete_states_need_update = true;
        while self.event_flags.discrete_states_need_update {
            self.inst
                .update_discrete_states(&mut self.event_flags)
                .map_err(Into::into)?;
            if self.event_flags.terminate_simulation {
                break;
            }
            reset_solver |= self.event_flags.nominals_of_continuous_states_changed
                || self.event_flags.values_of_continuous_states_changed;
        }
        Ok((reset_solver, self.event_flags.terminate_simulation))
    }
}

#[derive(Default, Debug)]
pub struct SimStats {
    /// End time of the simulation
    pub end_time: f64,
    /// Number of steps taken
    pub num_steps: usize,
    /// Number of events handled
    pub num_events: usize,
}

/// Lower-level simulation function that takes an FMI import and a set of options.
pub fn simulate_with<Imp: FmiSim>(
    input_data: Option<RecordBatch>,
    interface: &options::Interface,
    import: Imp,
) -> Result<(RecordBatch, SimStats), Error> {
    match interface {
        #[cfg(feature = "me")]
        options::Interface::ModelExchange(options) => import.simulate_me(options, input_data),
        #[cfg(feature = "cs")]
        options::Interface::CoSimulation(options) => import.simulate_cs(options, input_data),
        #[cfg(feature = "se")]
        options::Interface::ScheduledExecution(options) => unimplemented!(),
        #[cfg(any(not(feature = "me"), not(feature = "cs")))]
        _ => Err(fmi::Error::UnsupportedInterface(format!("{}", interface)).into()),
    }
}

macro_rules! impl_sim_default_initialize {
    ($inst:ty) => {
        impl SimDefaultInitialize for SimState<$inst> {
            fn default_initialize(&mut self) -> Result<(), Error> {
                self.inst
                    .enter_initialization_mode(
                        self.sim_params.tolerance,
                        self.sim_params.start_time,
                        Some(self.sim_params.stop_time),
                    )
                    .map_err(fmi::Error::from)?;

                self.inst
                    .exit_initialization_mode()
                    .map_err(fmi::Error::from)?;

                if self.sim_params.event_mode_used {
                    // update discrete states
                    self.event_flags.discrete_states_need_update = true;
                    while self.event_flags.discrete_states_need_update {
                        self.inst
                            .update_discrete_states(&mut self.event_flags)
                            .map_err(fmi::Error::from)?;

                        if self.event_flags.terminate_simulation {
                            self.inst.terminate().map_err(fmi::Error::from)?;
                            log::warn!("update_discrete_states() requested termination.");
                            break;
                        }
                    }
                }
                Ok(())
            }
        }
    };
}

#[cfg(feature = "me")]
impl_sim_default_initialize!(fmi::fmi2::instance::InstanceME<'_>);
#[cfg(feature = "cs")]
impl SimDefaultInitialize for SimState<fmi::fmi2::instance::InstanceCS<'_>> {
    fn default_initialize(&mut self) -> Result<(), Error> {
        self.inst
            .enter_initialization_mode(
                self.sim_params.tolerance,
                self.sim_params.start_time,
                Some(self.sim_params.stop_time),
            )
            .map_err(fmi::Error::from)?;
        self.inst
            .exit_initialization_mode()
            .map_err(fmi::Error::from)?;

        Ok(())
    }
}

#[cfg(feature = "me")]
impl_sim_default_initialize!(fmi::fmi3::instance::InstanceME<'_>);
#[cfg(feature = "cs")]
impl_sim_default_initialize!(fmi::fmi3::instance::InstanceCS<'_>);

macro_rules! impl_sim_initialize {
    ($inst:ty) => {
        impl traits::SimInitialize<$inst> for SimState<$inst> {
            fn initialize<P: AsRef<Path>>(
                &mut self,
                start_values: io::StartValues<<$inst as FmiInstance>::ValueRef>,
                initial_fmu_state_file: Option<P>,
            ) -> Result<(), Error> {
                if let Some(_initial_state_file) = &initial_fmu_state_file {
                    unimplemented!("initial_fmu_state_file");
                    // self.inst.restore_fmu_state_from_file(initial_state_file)?;
                }

                // set start values
                traits::SimApplyStartValues::apply_start_values(self, &start_values)?;

                self.input_state.apply_input::<interpolation::Linear>(
                    self.sim_params.start_time,
                    &mut self.inst,
                    true,
                    true,
                    false,
                )?;

                // Default initialization
                if initial_fmu_state_file.is_none() {
                    self.default_initialize()?;
                }

                Ok(())
            }
        }
    };
}

#[cfg(feature = "me")]
impl_sim_initialize!(fmi::fmi2::instance::InstanceME<'_>);
#[cfg(feature = "me")]
impl_sim_initialize!(fmi::fmi3::instance::InstanceME<'_>);
#[cfg(feature = "cs")]
impl_sim_initialize!(fmi::fmi2::instance::InstanceCS<'_>);
#[cfg(feature = "cs")]
impl_sim_initialize!(fmi::fmi3::instance::InstanceCS<'_>);
