use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct ModelExchange {
    /// Short class name according to C-syntax
    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,

    /// If true, a tool is needed to execute the model and the FMU just contains the communication
    /// to this tool.
    #[yaserde(attribute, rename = "needsExecutionTool")]
    pub needs_execution_tool: bool,

    #[yaserde(attribute, rename = "completedIntegratorStepNotNeeded")]
    pub completed_integrator_step_not_needed: bool,

    #[yaserde(attribute, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: bool,

    #[yaserde(attribute, rename = "canNotUseMemoryManagementFunctions")]
    pub can_not_use_memory_management_functions: bool,

    #[yaserde(attribute, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: bool,

    #[yaserde(attribute, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: bool,

    /// If true, the directional derivative of the equations can be computed with
    /// fmi2GetDirectionalDerivative
    #[yaserde(attribute, rename = "providesDirectionalDerivative")]
    pub provides_directional_derivative: bool,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct CoSimulation {
    /// Short class name according to C-syntax
    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,
}
