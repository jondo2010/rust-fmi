#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::too_many_arguments)]

pub mod meta;
pub mod instance;
pub mod logger;

use derive_more::Display;
/// Internal private low-level FMI types
use dlopen::wrapper::{WrapperApi, WrapperMultiApi};
use dlopen_derive::{WrapperApi, WrapperMultiApi};

pub const fmi2TypesPlatform: &[u8; 8usize] = b"default\0";
pub const fmi2True: fmi2Boolean = 1;
pub const fmi2False: fmi2Boolean = 0;

pub type fmi2Component = *mut ::std::os::raw::c_void;
pub type fmi2ComponentEnvironment = *mut ::std::os::raw::c_void;
pub type fmi2FMUstate = *mut ::std::os::raw::c_void;
pub type fmi2ValueReference = ::std::os::raw::c_uint;
pub type fmi2Real = std::os::raw::c_double;
pub type fmi2Integer = ::std::os::raw::c_long;
pub type fmi2Boolean = ::std::os::raw::c_int;
pub type fmi2Char = ::std::os::raw::c_char;
pub type fmi2String = *const fmi2Char;
pub type fmi2Byte = ::std::os::raw::c_char;

#[repr(C)]
#[derive(Debug, Display)]
pub enum fmi2Type {
    ModelExchange = 0,
    CoSimulation = 1,
}

#[repr(C)]
#[derive(Debug, Display)]
pub enum fmi2StatusKind {
    /// Can be called when the fmi2DoStep function returned fmi2Pending. The function delivers
    /// fmi2Pending if the computation is not finished. Otherwise the function returns the result
    /// of the asynchronously executed fmi2DoStep call
    DoStepStatus = 0,
    /// Can be called when the fmi2DoStep function returned fmi2Pending. The function delivers a
    /// string which informs about the status of the currently running asynchronous fmi2DoStep
    /// computation.
    PendingStatus = 1,
    /// Returns the end time of the last successfully completed communication step. Can be called
    /// after fmi2DoStep(...) returned fmi2Discard.
    LastSuccessfulTime = 2,
    /// Returns true, if the slave wants to terminate the simulation. Can be called after
    /// fmi2DoStep(...) returned fmi2Discard. Use fmi2LastSuccessfulTime to determine the time
    /// instant at which the slave terminated
    Terminated = 3,
}

#[repr(C)]
pub enum fmi2Status {
    /// All well
    OK = 0,
    /// things are not quite right, but the computation can continue. Function “logger” was called
    /// in the model (see below) and it is expected that this function has shown the prepared
    /// information message to the user
    Warning = 1,
    ///
    Discard = 2,
    Error = 3,
    Fatal = 4,
    /// Pending is returned only from the co-simulation interface, if the slave executes the
    /// function in an asynchronous way. That means the slave starts to compute but returns
    /// immediately. The master has to call fmi2GetStatus(..., fmi2DoStepStatus) to determine, if
    /// the slave has finished the computation. Can be returned only by `do_step` and by
    /// `get_status`
    Pending = 5,
}

type fmi2CallbackLogger = Option<
    unsafe extern "C" fn(
        component_environment: fmi2ComponentEnvironment,
        instance_name: fmi2String,
        status: fmi2Status,
        category: fmi2String,
        message: fmi2String,
        ...
    ),
>;

pub type fmi2CallbackAllocateMemory =
    Option<unsafe extern "C" fn(arg1: usize, arg2: usize) -> *mut libc::c_void>;

pub type fmi2CallbackFreeMemory = Option<unsafe extern "C" fn(arg1: *mut std::os::raw::c_void)>;

pub type fmi2StepFinished =
    Option<unsafe extern "C" fn(arg1: fmi2ComponentEnvironment, arg2: fmi2Status)>;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CallbackFunctions {
    pub logger: fmi2CallbackLogger,
    pub allocate_memory: fmi2CallbackAllocateMemory,
    pub free_memory: fmi2CallbackFreeMemory,
    pub step_finished: fmi2StepFinished,
    pub component_environment: fmi2ComponentEnvironment,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EventInfo {
    pub new_discrete_states_needed: fmi2Boolean,
    pub terminate_simulation: fmi2Boolean,
    pub nominals_of_continuous_states_changed: fmi2Boolean,
    pub values_of_continuous_states_changed: fmi2Boolean,
    pub next_event_time_defined: fmi2Boolean,
    pub next_event_time: fmi2Real,
}

/// Common API between ME and CS
#[derive(WrapperApi)]
pub struct Common {
    #[dlopen_name = "fmi2GetTypesPlatform"]
    get_types_platform: unsafe extern "C" fn() -> *const std::os::raw::c_char,

    #[dlopen_name = "fmi2GetVersion"]
    get_version: unsafe extern "C" fn() -> *const std::os::raw::c_char,

    #[dlopen_name = "fmi2SetDebugLogging"]
    set_debug_logging: unsafe extern "C" fn(
        c: fmi2Component,
        logging_on: fmi2Boolean,
        num_categories: usize,
        categories: *const fmi2String,
    ) -> fmi2Status,

    /// Wrapper for the FMI function fmiInstantiate(...)
    ///
    /// The function returns a new instance of an FMU. If a null pointer is returned, then
    /// instantiation failed. In that case, “functions->logger” was called with detailed
    /// information about the reason. An FMU can be instantiated many times (provided capability
    /// flag canBeInstantiatedOnlyOncePerProcess = false). This function must be called
    /// successfully, before any of the following functions can be called.
    ///
    /// For co-simulation, this function call has to perform all actions of a slave which are
    /// necessary before a simulation run starts (for example loading the model file,
    /// compilation...).
    ///
    /// * `instance_name` - a unique identifier for the FMU instance. It is used to name the
    ///   instance, for example in error or information messages generated by one of the fmi2XXX
    ///   functions. It is not allowed to provide a null pointer and this string must be non-empty
    ///   (in other words must have at least one character that is no white space). [If only one
    ///   FMU is simulated, as instanceName attribute modelName or <ModelExchange/CoSimulation
    ///   modelIdentifier=”..”> from the XML schema fmiModelDescription might be used.]
    /// * `fmu_type` - Argument fmuType defines the type of the FMU:
    ///     * fmi2ModelExchange: FMU with initialization and events;
    ///     * fmi2CoSimulation: Black box interface for co-simulation
    /// * `fmu_guid` - Argument fmuGUID is used to check that the modelDescription.xml file (see
    ///   section 2.3) is compatible with the C code of the FMU. It is a vendor specific globally
    ///   unique identifier of the XML file (for example it is a “fingerprint” of the relevant
    ///   information stored in the XML file). It is stored in the XML file as attribute “guid”
    ///   (see section 2.2.1) and has to be passed to the fmi2Instantiate function via argument
    ///   fmuGUID. It must be identical to the one stored inside the fmi2Instantiate function.
    ///   Otherwise the C code and the XML file of the FMU are not consistent to each other. This
    ///   argument cannot be null.
    /// * `fmu_resource_location` - Access path URI to the unzipped FMU archive resources. Argument
    ///   fmuResourceLocation is an URI according to the IETF RFC3986 syntax to indicate the
    ///   location to the “resources” directory of the unzipped FMU archive. The following schemes
    ///   must be understood by the FMU:
    ///     * Mandatory: “file” with absolute path (either including or omitting the authority
    ///       component)
    ///     * Optional: “http”, “https”, “ftp”
    ///     * Reserved: “fmi2” for FMI for PLM.
    ///     [Example: An FMU is unzipped in directory “C:\temp\MyFMU”, then fmuResourceLocation =
    ///     "file:///C:/temp/MyFMU/resources" or "file:/C:/temp/MyFMU/resources". Function
    ///     fmi2Instantiate is then able to read all needed resources from this directory, for
    ///     example maps or tables used by the FMU.]
    /// * `functions` -
    /// * `visible` -
    /// * `logging_on` -
    #[dlopen_name = "fmi2Instantiate"]
    instantiate: unsafe extern "C" fn(
        instance_name: fmi2String,
        fmu_type: fmi2Type,
        fmu_guid: fmi2String,
        fmu_resource_location: fmi2String,
        functions: *const CallbackFunctions,
        visible: fmi2Boolean,
        logging_on: fmi2Boolean,
    ) -> fmi2Component,

    #[dlopen_name = "fmi2FreeInstance"]
    free_instance: unsafe extern "C" fn(c: fmi2Component),

    #[dlopen_name = "fmi2SetupExperiment"]
    setup_experiment: unsafe extern "C" fn(
        c: fmi2Component,
        tolerance_defined: fmi2Boolean,
        tolerance: fmi2Real,
        start_time: fmi2Real,
        stop_time_defined: fmi2Boolean,
        stop_time: fmi2Real,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2EnterInitializationMode"]
    enter_initialization_mode: unsafe extern "C" fn(arg1: fmi2Component) -> fmi2Status,

    #[dlopen_name = "fmi2ExitInitializationMode"]
    exit_initialization_mode: unsafe extern "C" fn(arg1: fmi2Component) -> fmi2Status,

    #[dlopen_name = "fmi2Terminate"]
    terminate: unsafe extern "C" fn(arg1: fmi2Component) -> fmi2Status,

    #[dlopen_name = "fmi2Reset"]
    reset: unsafe extern "C" fn(arg1: fmi2Component) -> fmi2Status,

    #[dlopen_name = "fmi2GetReal"]
    get_real: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *mut fmi2Real,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetInteger"]
    get_integer: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *mut fmi2Integer,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetBoolean"]
    get_boolean: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *mut fmi2Boolean,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetString"]
    get_string: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *mut fmi2String,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2SetReal"]
    set_real: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *const fmi2Real,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2SetInteger"]
    set_integer: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *const fmi2Integer,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2SetBoolean"]
    set_boolean: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *const fmi2Boolean,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2SetString"]
    set_string: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        value: *const fmi2String,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetFMUstate"]
    get_fmu_state: unsafe extern "C" fn(c: fmi2Component, state: *mut fmi2FMUstate) -> fmi2Status,

    #[dlopen_name = "fmi2SetFMUstate"]
    set_fmu_state: unsafe extern "C" fn(c: fmi2Component, state: fmi2FMUstate) -> fmi2Status,

    #[dlopen_name = "fmi2FreeFMUstate"]
    free_fmu_state: unsafe extern "C" fn(c: fmi2Component, state: *mut fmi2FMUstate) -> fmi2Status,

    #[dlopen_name = "fmi2SerializedFMUstateSize"]
    serialized_fmu_state_size:
        unsafe extern "C" fn(c: fmi2Component, state: fmi2FMUstate, size: *mut usize) -> fmi2Status,

    #[dlopen_name = "fmi2SerializeFMUstate"]
    serialize_fmu_state: unsafe extern "C" fn(
        c: fmi2Component,
        state: fmi2FMUstate,
        bytes: *mut fmi2Byte,
        size: usize,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2DeSerializeFMUstate"]
    deserialize_fmu_state: unsafe extern "C" fn(
        c: fmi2Component,
        bytes: *const fmi2Byte,
        size: usize,
        state: *mut fmi2FMUstate,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetDirectionalDerivative"]
    get_directional_derivative: unsafe extern "C" fn(
        c: fmi2Component,
        vr1: *const fmi2ValueReference,
        size_vr1: usize,
        vr2: *const fmi2ValueReference,
        size_vr2: usize,
        x0: *const fmi2Real,
        der: *mut fmi2Real,
    ) -> fmi2Status,
}

/// Functions for FMI2 for Model Exchange
#[derive(WrapperApi)]
pub(crate) struct ME {
    // Enter and exit the different modes
    #[dlopen_name = "fmi2EnterEventMode"]
    enter_event_mode: unsafe extern "C" fn(c: fmi2Component) -> fmi2Status,

    #[dlopen_name = "fmi2NewDiscreteStates"]
    new_discrete_states:
        unsafe extern "C" fn(c: fmi2Component, event_info: *mut EventInfo) -> fmi2Status,

    #[dlopen_name = "fmi2EnterContinuousTimeMode"]
    enter_continuous_time_mode: unsafe extern "C" fn(c: fmi2Component) -> fmi2Status,

    #[dlopen_name = "fmi2CompletedIntegratorStep"]
    completed_integrator_step: unsafe extern "C" fn(
        c: fmi2Component,
        no_set_fmu_state_prior_to_current_point: fmi2Boolean,
        enter_event_mode: *mut fmi2Boolean,
        terminate_simulation: *mut fmi2Boolean,
    ) -> fmi2Status,

    // Providing independent variables and re-initialization of caching
    #[dlopen_name = "fmi2SetTime"]
    set_time: unsafe extern "C" fn(c: fmi2Component, time: fmi2Real) -> fmi2Status,

    /// Set a new (continuous) state vector and re-initialize caching of variables that depend on
    /// the states. Argument nx is the length of vector x and is provided for checking purposes
    /// (variables that depend solely on constants, parameters, time, and inputs do not need to be
    /// newly computed in the sequel, but the previously computed values can be reused).
    /// Note, the continuous states might also be changed in Event Mode.
    /// Note: fmi2Status = fmi2Discard is possible.
    #[dlopen_name = "fmi2SetContinuousStates"]
    set_continuous_states:
        unsafe extern "C" fn(c: fmi2Component, x: *const fmi2Real, nx: usize) -> fmi2Status,

    // Evaluation of the model equations
    #[dlopen_name = "fmi2GetDerivatives"]
    get_derivatives:
        unsafe extern "C" fn(c: fmi2Component, dx: *mut fmi2Real, nx: usize) -> fmi2Status,

    #[dlopen_name = "fmi2GetEventIndicators"]
    get_event_indicators: unsafe extern "C" fn(
        c: fmi2Component,
        event_indicators: *mut fmi2Real,
        ni: usize,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetContinuousStates"]
    get_continuous_states:
        unsafe extern "C" fn(c: fmi2Component, x: *mut fmi2Real, nx: usize) -> fmi2Status,

    #[dlopen_name = "fmi2GetNominalsOfContinuousStates"]
    get_nominals_of_continuous_states:
        unsafe extern "C" fn(c: fmi2Component, x_nominal: *mut fmi2Real, nx: usize) -> fmi2Status,
}

/// Functions for FMI2 for Co-Simulation
#[derive(WrapperApi)]
pub(crate) struct CS {
    // Simulating the slave
    #[dlopen_name = "fmi2SetRealInputDerivatives"]
    set_real_input_derivatives: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        order: *const fmi2Integer,
        value: *const fmi2Real,
    ) -> fmi2Status,

    /// Retrieves the n-th derivative of output values. Argument “vr” is a vector of “nvr” value
    /// references that define the variables whose derivatives shall be retrieved. The array
    /// “order” contains the order of the respective derivative (1 means the first derivative, 0
    /// is not allowed). Argument “value” is a vector with the actual values of the derivatives.
    /// Restrictions on using the function are the same as for the fmi2GetReal function.
    #[dlopen_name = "fmi2GetRealOutputDerivatives"]
    get_real_input_derivatives: unsafe extern "C" fn(
        c: fmi2Component,
        vr: *const fmi2ValueReference,
        nvr: usize,
        order: *const fmi2Integer,
        value: *mut fmi2Real,
    ) -> fmi2Status,

    /// The computation of a time step is started.
    #[dlopen_name = "fmi2DoStep"]
    do_step: unsafe extern "C" fn(
        c: fmi2Component,
        current_communication_point: fmi2Real,
        communication_step_size: fmi2Real,
        new_step: fmi2Boolean,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2CancelStep"]
    cancel_step: unsafe extern "C" fn(c: fmi2Component) -> fmi2Status,

    // Inquire slave status
    #[dlopen_name = "fmi2GetStatus"]
    get_status: unsafe extern "C" fn(
        c: fmi2Component,
        s: fmi2StatusKind,
        value: *mut fmi2Status,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetRealStatus"]
    get_real_status: unsafe extern "C" fn(
        c: fmi2Component,
        s: fmi2StatusKind,
        value: *mut fmi2Real,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetIntegerStatus"]
    get_integer_status: unsafe extern "C" fn(
        c: fmi2Component,
        s: fmi2StatusKind,
        value: *mut fmi2Integer,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetBooleanStatus"]
    get_boolean_status: unsafe extern "C" fn(
        c: fmi2Component,
        s: fmi2StatusKind,
        value: *mut fmi2Boolean,
    ) -> fmi2Status,

    #[dlopen_name = "fmi2GetStringStatus"]
    get_string_status: unsafe extern "C" fn(
        c: fmi2Component,
        s: fmi2StatusKind,
        value: *mut fmi2String,
    ) -> fmi2Status,
}

pub trait FmiApi: WrapperApi {
    fn common(&self) -> &Common;
}

#[derive(WrapperMultiApi)]
pub struct Fmi2ME {
    pub(crate) common: Common,
    pub(crate) me: ME,
}

impl FmiApi for Fmi2ME {
    fn common(&self) -> &Common {
        &self.common
    }
}

#[derive(WrapperMultiApi)]
pub struct Fmi2CS {
    pub(crate) common: Common,
    pub(crate) cs: CS,
}

impl FmiApi for Fmi2CS {
    fn common(&self) -> &Common {
        &self.common
    }
}
