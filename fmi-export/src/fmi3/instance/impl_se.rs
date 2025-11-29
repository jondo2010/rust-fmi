use fmi::fmi3::ScheduledExecution;

use crate::fmi3::{Context, Model, ModelGetSet, ModelGetSetStates, ModelInstance, UserModel};

impl<M, C> ScheduledExecution for ModelInstance<M, C>
where
    M: Model + UserModel + ModelGetSet<M> + ModelGetSetStates,
    C: Context<M>,
{
    fn activate_model_partition(
        &mut self,
        _clock_reference: fmi::fmi3::binding::fmi3ValueReference,
        _activation_time: f64,
    ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
        //TODO: implement activate_model_partition. For now, report and return Error.
        self.context.log(
            fmi::fmi3::Fmi3Error::Error.into(),
            M::LoggingCategory::default(),
            format_args!("activate_model_partition() is not implemented yet."),
        );
        Err(fmi::fmi3::Fmi3Error::Error)
    }
}
