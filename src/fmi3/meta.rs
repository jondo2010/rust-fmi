use strong_xml::XmlRead;

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "fmiModelDescription")]
pub struct ModelDescription {
    /// Version of FMI that was used to generate the XML file.
    #[xml(attr = "fmiVersion")]
    pub fmi_version: String,
    /// The name of the model as used in the modeling environment that generated the XML file, such as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[xml(attr = "modelName")]
    pub model_name: String,
    /// Optional string with a brief description of the model.
    #[xml(attr = "description")]
    pub description: String,
    /// The instantiationToken is a string that can be used by the FMU to check that the XML file is compatible with the implementation of the FMU.
    #[xml(attr = "instantiationToken")]
    pub instantiation_token: String,
    /// If present, the FMU is based on FMI for Model Exchange
    #[xml(child = "ModelExchange")]
    pub model_exchange: Option<ModelExchange>,
    /// If present, the FMU is based on FMI for Co-Simulation
    #[xml(child = "CoSimulation")]
    pub co_simulation: Option<CoSimulation>,
    /// If present, the FMU is based on FMI for Scheduled Execution
    #[xml(child = "ScheduledExecution")]
    pub scheduled_execution: Option<ScheduledExecution>,
    /// Providing default settings for the integrator, such as stop time and relative tolerance.
    #[xml(child = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,
    /// A global list of unit and display unit definitions
    #[xml(child = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,
    /// A global list of type definitions that are utilized in `ModelVariables`
    #[xml(child = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "ModelExchange")]
pub struct ModelExchange {
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,
    #[xml(attr = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: bool,
    #[xml(attr = "canSerializeFMUState")]
    pub can_serialize_fmu_state: bool,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "CoSimulation")]
pub struct CoSimulation {
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,
    #[xml(attr = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: bool,
    #[xml(attr = "canSerializeFMUState")]
    pub can_serialize_fmu_state: bool,
    #[xml(attr = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: bool,
    #[xml(attr = "providesIntermediateUpdate")]
    pub provides_intermediate_update: bool,
    #[xml(attr = "canReturnEarlyAfterIntermediateUpdate")]
    pub can_return_early_after_intermediate_update: bool,
    #[xml(attr = "fixedInternalStepSize")]
    pub fixed_internal_step_size: f64,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "ScheduledExecution")]
pub struct ScheduledExecution {
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "DefaultExperiment")]
pub struct DefaultExperiment {
    #[xml(attr = "startTime")]
    pub start_time: Option<f64>,
    #[xml(attr = "stopTime")]
    pub stop_time: Option<f64>,
    #[xml(attr = "tolerance")]
    pub tolerange: Option<f64>,
    #[xml(attr = "stepSize")]
    pub step_size: Option<f64>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "UnitDefinitions")]
pub struct UnitDefinitions {
    #[xml(child = "Unit")]
    pub units: Vec<Unit>,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "Unit")]
pub struct Unit {
    #[xml(attr = "name")]
    pub name: String,
}

#[derive(XmlRead, PartialEq, Debug)]
#[xml(tag = "TypeDefinitions")]
pub struct TypeDefinitions {
    #[xml(child = "Float64Type")]
    pub types: Vec<Type>,
}

#[derive(XmlRead, PartialEq, Debug)]
pub enum Type {
    #[xml(tag = "Float64Type")]
    Float64Type {
        #[xml(attr = "name")]
        name: String,
        #[xml(attr = "description")]
        description: Option<String>,
        #[xml(attr = "quantity")]
        quantity: Option<String>,
        #[xml(attr = "unit")]
        unit: Option<String>,
    },
}
