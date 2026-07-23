//! End-to-end test of the FMI 3.0 **Scheduled Execution** machinery through the
//! generated C ABI: instantiate SE → init → activate model partition →
//! get countdown interval → read outputs → free.
//!
//! This exercises exactly the code paths that used to be `todo!()`/`Err`:
//! `fmi3InstantiateScheduledExecution`, `fmi3ActivateModelPartition`,
//! `fmi3GetIntervalDecimal` (with the "observe once per tick" qualifier rule),
//! `fmi3GetIntervalFraction`, `fmi3GetShiftDecimal`, `fmi3EvaluateDiscreteStates`
//! and `fmi3FreeInstance` for an SE instance.

use std::ffi::CString;

use fmi::{
    fmi3::{binding, Fmi3Error, Fmi3Res, Fmi3Status},
    traits::FmiStatus,
};
use fmi_export::{
    fmi3::{Clock, Context, DefaultLoggingCategory, Fmi3Common, Fmi3ScheduledExecution, Model, UserModel},
    FmuModel,
};

// A minimal aperiodic (DEVS) traffic light, SE-only, mirroring `semaforo_se`.
//   VR1 = boton (input), VR2 = reloj (countdown input clock),
//   VR3 = rojo, VR4 = verde, VR5 = t_restante (outputs)
#[derive(FmuModel, Debug)]
#[model(
    model_exchange = false,
    co_simulation = false,
    scheduled_execution = true,
    user_model = false
)]
struct TrafficSE {
    #[variable(causality = Input, variability = Discrete, start = 0.0)]
    boton: f64,
    #[variable(causality = Input, interval_variability = Countdown)]
    reloj: Clock,
    #[variable(causality = Output, variability = Discrete, start = 1.0, clocks = [reloj])]
    rojo: f64,
    #[variable(causality = Output, variability = Discrete, start = 0.0, clocks = [reloj])]
    verde: f64,
    #[variable(causality = Output, variability = Discrete, start = 60.0, clocks = [reloj])]
    t_restante: f64,

    en_rojo: bool,
    sigma: f64,
}

impl Default for TrafficSE {
    fn default() -> Self {
        Self {
            boton: 0.0,
            reloj: Clock::default(),
            rojo: 1.0,
            verde: 0.0,
            t_restante: 60.0,
            en_rojo: true,
            sigma: 60.0,
        }
    }
}

impl UserModel for TrafficSE {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        self.rojo = if self.en_rojo { 1.0 } else { 0.0 };
        self.verde = if self.en_rojo { 0.0 } else { 1.0 };
        self.t_restante = self.sigma;
        Ok(Fmi3Res::OK)
    }

    fn activate_partition(
        &mut self,
        _context: &mut dyn Context<Self>,
        _clock_reference: binding::fmi3ValueReference,
        _time: f64,
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Toggle phase and reset the countdown to the new phase's duration.
        self.en_rojo = !self.en_rojo;
        self.sigma = if self.en_rojo { 60.0 } else { 30.0 };
        Ok(Fmi3Res::OK)
    }

    fn next_interval(&self, _clock_reference: binding::fmi3ValueReference) -> Option<f64> {
        Some(self.sigma)
    }
}

const VR_RELOJ: binding::fmi3ValueReference = 2;
const VR_ROJO: binding::fmi3ValueReference = 3;
const VR_VERDE: binding::fmi3ValueReference = 4;
const VR_T_RESTANTE: binding::fmi3ValueReference = 5;

fn instantiate_se() -> binding::fmi3Instance {
    unsafe {
        <TrafficSE as Fmi3Common>::fmi3_instantiate_scheduled_execution(
            CString::new("traffic").unwrap().as_ptr(),
            CString::new(TrafficSE::INSTANTIATION_TOKEN).unwrap().as_ptr() as *mut i8,
            CString::new("path/to/fmu").unwrap().as_ptr(),
            false as _, // visible
            true as _,  // logging_on
            std::ptr::null_mut(),
            None, // log_message
            None, // clock_update
            None, // lock_preemption
            None, // unlock_preemption
        )
    }
}

fn get_interval(inst: binding::fmi3Instance) -> (f64, binding::fmi3IntervalQualifier) {
    let mut interval = [0.0f64; 1];
    let mut qual = [0 as binding::fmi3IntervalQualifier; 1];
    let status = Fmi3Status::from(unsafe {
        <TrafficSE as Fmi3Common>::fmi3_get_interval_decimal(
            inst,
            [VR_RELOJ].as_ptr(),
            1,
            interval.as_mut_ptr(),
            qual.as_mut_ptr(),
        )
    })
    .ok();
    assert_eq!(status, Ok(Fmi3Res::OK));
    (interval[0], qual[0])
}

fn get_f64(inst: binding::fmi3Instance, vr: binding::fmi3ValueReference) -> f64 {
    let mut val = [0.0f64; 1];
    let status = Fmi3Status::from(unsafe {
        <TrafficSE as Fmi3Common>::fmi3_get_float64(inst, [vr].as_ptr(), 1, val.as_mut_ptr(), 1)
    })
    .ok();
    assert_eq!(status, Ok(Fmi3Res::OK));
    val[0]
}

fn activate(inst: binding::fmi3Instance, t: f64) -> Result<Fmi3Res, Fmi3Error> {
    Fmi3Status::from(unsafe {
        <TrafficSE as Fmi3ScheduledExecution>::fmi3_activate_model_partition(inst, VR_RELOJ, t)
    })
    .ok()
}

#[test]
fn se_full_lifecycle_through_c_abi() {
    let inst = instantiate_se();
    assert!(!inst.is_null(), "fmi3InstantiateScheduledExecution returned null");

    // Init.
    assert_eq!(
        Fmi3Status::from(unsafe {
            <TrafficSE as Fmi3Common>::fmi3_enter_initialization_mode(inst, false, 0.0, 0.0, false, 0.0)
        })
        .ok(),
        Ok(Fmi3Res::OK)
    );
    assert_eq!(
        Fmi3Status::from(unsafe {
            <TrafficSE as Fmi3Common>::fmi3_exit_initialization_mode(inst)
        })
        .ok(),
        Ok(Fmi3Res::OK)
    );

    // Initial outputs: RED, 60 s remaining.
    assert_eq!(get_f64(inst, VR_ROJO), 1.0);
    assert_eq!(get_f64(inst, VR_VERDE), 0.0);
    assert_eq!(get_f64(inst, VR_T_RESTANTE), 60.0);

    // The scheduler reads the initial countdown interval: Changed, 60 s.
    let (interval, qual) = get_interval(inst);
    assert_eq!(interval, 60.0);
    assert_eq!(qual, binding::fmi3IntervalQualifier_fmi3IntervalChanged);

    // "Observe once per tick": a second read with no activation → Unchanged.
    let (_interval, qual2) = get_interval(inst);
    assert_eq!(qual2, binding::fmi3IntervalQualifier_fmi3IntervalUnchanged);

    // Scheduler activates the partition at t = 60 (end of red).
    assert_eq!(activate(inst, 60.0), Ok(Fmi3Res::OK));

    // Now GREEN, 30 s remaining, and the interval is Changed again.
    assert_eq!(get_f64(inst, VR_ROJO), 0.0);
    assert_eq!(get_f64(inst, VR_VERDE), 1.0);
    let (interval, qual) = get_interval(inst);
    assert_eq!(interval, 30.0);
    assert_eq!(qual, binding::fmi3IntervalQualifier_fmi3IntervalChanged);

    // Another activation at t = 90 (end of green) → back to RED, 60 s.
    assert_eq!(activate(inst, 90.0), Ok(Fmi3Res::OK));
    assert_eq!(get_f64(inst, VR_ROJO), 1.0);
    let (interval, _q) = get_interval(inst);
    assert_eq!(interval, 60.0);

    // fmi3EvaluateDiscreteStates must be sound (no longer a todo!()).
    assert_eq!(
        Fmi3Status::from(unsafe {
            <TrafficSE as Fmi3Common>::fmi3_evaluate_discrete_states(inst)
        })
        .ok(),
        Ok(Fmi3Res::OK)
    );

    // Clean up (this path used to leak the instance).
    unsafe { <TrafficSE as Fmi3Common>::fmi3_free_instance(inst) };
}

#[test]
fn se_interval_fraction_and_shift() {
    let inst = instantiate_se();
    assert!(!inst.is_null());
    unsafe {
        <TrafficSE as Fmi3Common>::fmi3_enter_initialization_mode(inst, false, 0.0, 0.0, false, 0.0);
        <TrafficSE as Fmi3Common>::fmi3_exit_initialization_mode(inst);
    }

    // fmi3GetIntervalFraction: 60 s == 60 * 1e9 / 1e9.
    let mut counter = [0u64; 1];
    let mut resolution = [0u64; 1];
    let mut qual = [0 as binding::fmi3IntervalQualifier; 1];
    let status = Fmi3Status::from(unsafe {
        <TrafficSE as Fmi3Common>::fmi3_get_interval_fraction(
            inst,
            [VR_RELOJ].as_ptr(),
            1,
            counter.as_mut_ptr(),
            resolution.as_mut_ptr(),
            qual.as_mut_ptr(),
        )
    })
    .ok();
    assert_eq!(status, Ok(Fmi3Res::OK));
    assert_eq!(qual[0], binding::fmi3IntervalQualifier_fmi3IntervalChanged);
    // counter / resolution == 60.0
    assert_eq!(counter[0] as f64 / resolution[0] as f64, 60.0);

    // fmi3GetShiftDecimal: unshifted clock → 0.0.
    let mut shift = [123.0f64; 1];
    let status = Fmi3Status::from(unsafe {
        <TrafficSE as Fmi3Common>::fmi3_get_shift_decimal(
            inst,
            [VR_RELOJ].as_ptr(),
            1,
            shift.as_mut_ptr(),
        )
    })
    .ok();
    assert_eq!(status, Ok(Fmi3Res::OK));
    assert_eq!(shift[0], 0.0);

    unsafe { <TrafficSE as Fmi3Common>::fmi3_free_instance(inst) };
}

#[test]
fn se_activate_before_init_is_rejected() {
    // Activating a partition outside Clock Activation Mode must fail cleanly (not panic).
    let inst = instantiate_se();
    assert!(!inst.is_null());
    // No enter/exit init → still in Instantiated state.
    assert_eq!(activate(inst, 1.0), Err(Fmi3Error::Error));
    unsafe { <TrafficSE as Fmi3Common>::fmi3_free_instance(inst) };
}

// A multi-partition SE model with three countdown clocks. clock_a (VR1) and clock_b (VR2)
// have known intervals; clock_c (VR3) never reports one (models "not yet known").
#[derive(FmuModel, Debug)]
#[model(
    model_exchange = false,
    co_simulation = false,
    scheduled_execution = true,
    user_model = false
)]
struct MultiClock {
    #[variable(causality = Input, interval_variability = Countdown)]
    clock_a: Clock,
    #[variable(causality = Input, interval_variability = Countdown)]
    clock_b: Clock,
    #[variable(causality = Input, interval_variability = Countdown)]
    clock_c: Clock,
    #[variable(causality = Output, variability = Discrete, start = 0.0, clocks = [clock_a])]
    ticks_a: f64,
    #[variable(causality = Output, variability = Discrete, start = 0.0, clocks = [clock_b])]
    ticks_b: f64,

    sigma_a: f64,
    sigma_b: f64,
}

impl Default for MultiClock {
    fn default() -> Self {
        Self {
            clock_a: Clock::default(),
            clock_b: Clock::default(),
            clock_c: Clock::default(),
            ticks_a: 0.0,
            ticks_b: 0.0,
            sigma_a: 10.0,
            sigma_b: 20.0,
        }
    }
}

impl UserModel for MultiClock {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _c: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }

    fn activate_partition(
        &mut self,
        _c: &mut dyn Context<Self>,
        clock_reference: binding::fmi3ValueReference,
        _time: f64,
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Dispatch on the input Clock: only the ticked partition advances.
        match clock_reference {
            1 => self.ticks_a += 1.0,
            2 => self.ticks_b += 1.0,
            _ => {}
        }
        Ok(Fmi3Res::OK)
    }

    fn next_interval(&self, clock_reference: binding::fmi3ValueReference) -> Option<f64> {
        match clock_reference {
            1 => Some(self.sigma_a),
            2 => Some(self.sigma_b),
            _ => None, // clock_c: interval not yet known
        }
    }
}

fn mc_get_interval(
    inst: binding::fmi3Instance,
    vr: binding::fmi3ValueReference,
) -> (f64, binding::fmi3IntervalQualifier) {
    let mut interval = [0.0f64; 1];
    let mut qual = [0 as binding::fmi3IntervalQualifier; 1];
    let status = Fmi3Status::from(unsafe {
        <MultiClock as Fmi3Common>::fmi3_get_interval_decimal(
            inst,
            [vr].as_ptr(),
            1,
            interval.as_mut_ptr(),
            qual.as_mut_ptr(),
        )
    })
    .ok();
    assert_eq!(status, Ok(Fmi3Res::OK));
    (interval[0], qual[0])
}

/// The interval qualifier must be tracked **per Clock**: activating clock A must not make
/// clock B look `Changed`, reading A must not clear B, and a clock with no interval must
/// report `NotYetKnown` (not `Unchanged`).
#[test]
fn se_multiclock_qualifiers_are_per_clock() {
    const VR_A: binding::fmi3ValueReference = 1;
    const VR_B: binding::fmi3ValueReference = 2;
    const VR_C: binding::fmi3ValueReference = 3;

    let inst = unsafe {
        <MultiClock as Fmi3Common>::fmi3_instantiate_scheduled_execution(
            CString::new("multi").unwrap().as_ptr(),
            CString::new(MultiClock::INSTANTIATION_TOKEN).unwrap().as_ptr() as *mut i8,
            CString::new("path").unwrap().as_ptr(),
            false as _,
            true as _,
            std::ptr::null_mut(),
            None,
            None,
            None,
            None,
        )
    };
    assert!(!inst.is_null());
    unsafe {
        <MultiClock as Fmi3Common>::fmi3_enter_initialization_mode(inst, false, 0.0, 0.0, false, 0.0);
        <MultiClock as Fmi3Common>::fmi3_exit_initialization_mode(inst);
    }

    let changed = binding::fmi3IntervalQualifier_fmi3IntervalChanged;
    let unchanged = binding::fmi3IntervalQualifier_fmi3IntervalUnchanged;
    let not_yet = binding::fmi3IntervalQualifier_fmi3IntervalNotYetKnown;

    // Initial reads: each clock reports its interval as Changed once.
    assert_eq!(mc_get_interval(inst, VR_A), (10.0, changed));
    assert_eq!(mc_get_interval(inst, VR_A).1, unchanged); // observed twice → Unchanged
    assert_eq!(mc_get_interval(inst, VR_B), (20.0, changed)); // B independent of A

    // Activate ONLY clock A.
    assert_eq!(
        Fmi3Status::from(unsafe {
            <MultiClock as Fmi3ScheduledExecution>::fmi3_activate_model_partition(inst, VR_A, 10.0)
        })
        .ok(),
        Ok(Fmi3Res::OK)
    );

    // B did NOT tick → still Unchanged. (A single global flag would wrongly say Changed.)
    assert_eq!(mc_get_interval(inst, VR_B).1, unchanged);
    // A ticked → Changed again.
    assert_eq!(mc_get_interval(inst, VR_A), (10.0, changed));

    // clock_c never reports an interval → NotYetKnown (never Unchanged).
    assert_eq!(mc_get_interval(inst, VR_C).1, not_yet);

    unsafe { <MultiClock as Fmi3Common>::fmi3_free_instance(inst) };
}
