//! FMI 3.0 **Scheduled Execution** semantics for [`ModelInstance`].
//!
//! This module is to Scheduled Execution what [`super::impl_cs`] is to Co-Simulation.
//! It provides:
//!
//! 1. The [`ScheduledExecution`] trait impl (`activate_model_partition`), the SE
//!    analogue of `do_step`.
//! 2. The instance-level clock-interval helpers that the raw C wrappers in
//!    [`crate::fmi3::traits::wrappers`] dispatch to (`get_interval_decimal`,
//!    `get_interval_fraction`, `get_shift_decimal`, `get_shift_fraction`,
//!    `evaluate_discrete_states`). These are inherent methods so that the generic
//!    `dispatch_by_instance_type!` macro can call them on any `ModelInstance`,
//!    regardless of interface type.
//!
//! ## Countdown Clocks (aperiodic partitions)
//!
//! An SE model is split into *model partitions*, each bound to an **input Clock**.
//! The scheduler activates a partition with `fmi3ActivateModelPartition(clock, t)`.
//! For a Clock with `intervalVariability = "countdown"`, the FMU itself decides *when*
//! the next tick happens: after each activation the scheduler reads
//! `fmi3GetIntervalDecimal` to obtain the remaining interval `σ` (the DEVS `ta()`),
//! then schedules the next activation `σ` seconds later. That pull-based loop —
//! `activate → getInterval → schedule → activate → …` — is the whole aperiodic
//! (DEVS) mechanism, and it is why countdown Clocks report their interval through the
//! `UserModel::next_interval` hook rather than through the `fmi3ClockUpdateCallback`.
//!
//! See <https://fmi-standard.org/docs/3.0.1/#fmi-for-scheduled-execution>.

use fmi::fmi3::{Fmi3Error, Fmi3Res, ScheduledExecution, binding};

use crate::fmi3::{
    Context, Model, ModelGetSetStates, ModelInstance, ModelState, UserModel,
    traits::{ModelGetSet, ModelLoggingCategory},
};

/// Resolution (ticks per second) used to express decimal countdown intervals as the
/// rational `counter / resolution` demanded by `fmi3GetIntervalFraction`.
///
/// `1_000_000_000` gives nanosecond granularity, which is far finer than any interval
/// our discrete models produce, so the decimal → fraction conversion is loss-free in
/// practice. Only relevant for importers that read the *fraction* form of the interval;
/// importers of a purely decimal countdown Clock use `fmi3GetIntervalDecimal` instead.
const INTERVAL_FRACTION_RESOLUTION: u64 = 1_000_000_000;

impl<M, C> ScheduledExecution for ModelInstance<M, C>
where
    M: Model + UserModel + ModelGetSet<M> + ModelGetSetStates,
    C: Context<M>,
{
    /// Activate the model partition triggered by the input Clock `clock_reference`
    /// at simulation time `activation_time`.
    ///
    /// Mirrors [`super::impl_cs`]'s `do_step`: validate the interface/state, advance
    /// the instance clock, run the user's partition transition, then mark outputs
    /// dirty so the next getter recomputes them, and flag that a fresh countdown
    /// interval is now available.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#fmi3ActivateModelPartition>.
    fn activate_model_partition(
        &mut self,
        clock_reference: binding::fmi3ValueReference,
        activation_time: f64,
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK.into(),
            M::LoggingCategory::trace_category(),
            format_args!(
                "activate_model_partition(clock: {clock_reference}, t: {activation_time})"
            ),
        );
        self.assert_instance_type(fmi::InterfaceType::ScheduledExecution)?;

        // Partitions may only be activated in Clock Activation Mode (entered on
        // `exit_initialization_mode` for an SE instance — see `common.rs`).
        match self.state {
            ModelState::ClockActivationMode => {}
            _ => {
                self.context.log(
                    Fmi3Error::Error.into(),
                    M::LoggingCategory::default(),
                    format_args!(
                        "activate_model_partition() called in invalid state {:?}",
                        self.state
                    ),
                );
                return Err(Fmi3Error::Error);
            }
        }

        // The tick happens at the Clock's activation time.
        self.context.set_time(activation_time);

        // Run the user's partition transition (δint/δext + λ in DEVS terms).
        let res = self
            .model
            .activate_partition(&mut self.context, clock_reference, activation_time)?;

        // Outputs must be recomputed on the next getter. This Clock just ticked, so its
        // interval is `Changed` again: drop it from the "already observed" set (only this
        // Clock — other partitions are unaffected).
        self.is_dirty_values = true;
        self.intervals_observed.remove(&clock_reference);

        Ok(res)
    }
}

/// Inherent SE clock helpers dispatched to from the raw C wrappers.
///
/// These are defined on every `ModelInstance` (not only SE ones) because the
/// `dispatch_by_instance_type!` macro resolves the method for all interface types.
/// For non-SE models the underlying `UserModel::next_interval` simply returns `None`,
/// so the helpers are inert.
impl<M, C> ModelInstance<M, C>
where
    M: Model + UserModel + ModelGetSet<M> + ModelGetSetStates,
    C: Context<M>,
{
    /// Backs `fmi3GetIntervalDecimal`: report each Clock's next interval in seconds,
    /// together with the interval qualifier.
    ///
    /// Qualifier rules, tracked **per Clock** (see [`ModelInstance::intervals_observed`]):
    /// - model returns `Some(σ)`, first read since the Clock's last tick → `fmi3IntervalChanged`;
    /// - model returns `Some(σ)`, already observed, no tick since → `fmi3IntervalUnchanged`;
    /// - model returns `None` (no interval yet) → `fmi3IntervalNotYetKnown`.
    pub(crate) fn get_interval_decimal(
        &mut self,
        clock_refs: &[binding::fmi3ValueReference],
        intervals: &mut [binding::fmi3Float64],
        qualifiers: &mut [binding::fmi3IntervalQualifier],
    ) -> Result<Fmi3Res, Fmi3Error> {
        for (i, &vr) in clock_refs.iter().enumerate() {
            let qualifier = match self.model.next_interval(vr) {
                Some(interval) => {
                    if let Some(slot) = intervals.get_mut(i) {
                        *slot = interval;
                    }
                    // `insert` returns true iff `vr` was NOT already observed → first read
                    // since the last tick → Changed; otherwise Unchanged.
                    if self.intervals_observed.insert(vr) {
                        binding::fmi3IntervalQualifier_fmi3IntervalChanged
                    } else {
                        binding::fmi3IntervalQualifier_fmi3IntervalUnchanged
                    }
                }
                None => binding::fmi3IntervalQualifier_fmi3IntervalNotYetKnown,
            };
            if let Some(slot) = qualifiers.get_mut(i) {
                *slot = qualifier;
            }
        }
        Ok(Fmi3Res::OK)
    }

    /// Backs `fmi3GetIntervalFraction`: the same information as
    /// [`Self::get_interval_decimal`] expressed as the rational `counter / resolution`
    /// (see [`INTERVAL_FRACTION_RESOLUTION`]).
    pub(crate) fn get_interval_fraction(
        &mut self,
        clock_refs: &[binding::fmi3ValueReference],
        counters: &mut [binding::fmi3UInt64],
        resolutions: &mut [binding::fmi3UInt64],
        qualifiers: &mut [binding::fmi3IntervalQualifier],
    ) -> Result<Fmi3Res, Fmi3Error> {
        for (i, &vr) in clock_refs.iter().enumerate() {
            let qualifier = match self.model.next_interval(vr) {
                Some(interval) => {
                    let counter =
                        (interval.max(0.0) * INTERVAL_FRACTION_RESOLUTION as f64).round() as u64;
                    if let Some(slot) = counters.get_mut(i) {
                        *slot = counter;
                    }
                    if let Some(slot) = resolutions.get_mut(i) {
                        *slot = INTERVAL_FRACTION_RESOLUTION;
                    }
                    if self.intervals_observed.insert(vr) {
                        binding::fmi3IntervalQualifier_fmi3IntervalChanged
                    } else {
                        binding::fmi3IntervalQualifier_fmi3IntervalUnchanged
                    }
                }
                None => binding::fmi3IntervalQualifier_fmi3IntervalNotYetKnown,
            };
            if let Some(slot) = qualifiers.get_mut(i) {
                *slot = qualifier;
            }
        }
        Ok(Fmi3Res::OK)
    }

    /// Backs `fmi3GetShiftDecimal`. The Clock *shift* is a constant phase offset applied
    /// before the first tick. Models built with `fmi-export` do not use shifted Clocks,
    /// so every shift is reported as `0.0` (no offset), which is the correct value for an
    /// unshifted Clock. Non-zero shifts would require per-Clock storage that no current
    /// model exercises.
    pub(crate) fn get_shift_decimal(
        &mut self,
        clock_refs: &[binding::fmi3ValueReference],
        shifts: &mut [binding::fmi3Float64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        for i in 0..clock_refs.len() {
            if let Some(slot) = shifts.get_mut(i) {
                *slot = 0.0;
            }
        }
        Ok(Fmi3Res::OK)
    }

    /// Backs `fmi3GetShiftFraction`: the fractional form of [`Self::get_shift_decimal`].
    /// A zero shift is `0 / 1`.
    pub(crate) fn get_shift_fraction(
        &mut self,
        clock_refs: &[binding::fmi3ValueReference],
        counters: &mut [binding::fmi3UInt64],
        resolutions: &mut [binding::fmi3UInt64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        for i in 0..clock_refs.len() {
            if let Some(slot) = counters.get_mut(i) {
                *slot = 0;
            }
            if let Some(slot) = resolutions.get_mut(i) {
                *slot = 1;
            }
        }
        Ok(Fmi3Res::OK)
    }

    /// Backs `fmi3EvaluateDiscreteStates`: bring the model's calculated values (outputs,
    /// discrete states) up to date. We reuse the same lazy-evaluation path as the value
    /// getters: recompute via `calculate_values` only if the model is dirty.
    pub(crate) fn evaluate_discrete_states(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK.into(),
            M::LoggingCategory::trace_category(),
            format_args!("evaluate_discrete_states()"),
        );
        if self.is_dirty_values {
            self.model.calculate_values(&self.context)?;
            self.is_dirty_values = false;
        }
        Ok(Fmi3Res::OK)
    }
}
