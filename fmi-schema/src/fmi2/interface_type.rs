use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
#[yaserde(tag = "File")]
pub struct File {
    /// Name of the file including the path relative to the sources directory, using the forward slash as separator
    /// (for example: name = "myFMU.c"; name = "modelExchange/solve.c")
    #[yaserde(attribute)]
    pub name: String,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
#[yaserde(tag = "SourceFiles")]
pub struct SourceFiles {
    #[yaserde(child, rename = "File")]
    pub files: Vec<File>,
}

/// The FMU includes a model or the communication to a tool that provides a model. The environment provides the
/// simulation engine for the model.
#[derive(Default, Debug, YaSerialize, YaDeserialize)]
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

    #[yaserde(attribute, rename = "canGetAndSetFMUstate")]
    pub can_get_and_set_fmu_state: bool,

    #[yaserde(attribute, rename = "canSerializeFMUstate")]
    pub can_serialize_fmu_state: bool,

    /// If true, the directional derivative of the equations can be computed with
    /// fmi2GetDirectionalDerivative
    #[yaserde(attribute, rename = "providesDirectionalDerivative")]
    pub provides_directional_derivative: bool,

    /// List of source file names that are present in the "sources" directory of the FMU and need to be compiled in
    /// order to generate the binary of the FMU (only meaningful for source code FMUs).
    #[yaserde(child, rename = "SourceFiles")]
    pub source_files: SourceFiles,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct CoSimulation {
    /// Short class name according to C-syntax
    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,

    /// If true, a tool is needed to execute the model and the FMU just contains the communication to this tool.
    #[yaserde(attribute, rename = "needsExecutionTool")]
    pub needs_execution_tool: bool,

    #[yaserde(attribute, rename = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: bool,

    #[yaserde(attribute, rename = "canInterpolateInputs")]
    pub can_interpolate_inputs: bool,

    #[yaserde(attribute, rename = "maxOutputDerivativeOrder")]
    pub max_output_derivative_order: u32,

    #[yaserde(attribute, rename = "canRunAsynchronuously")]
    pub can_run_asynchronuously: bool,

    #[yaserde(attribute, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: bool,

    #[yaserde(attribute, rename = "canNotUseMemoryManagementFunctions")]
    pub can_not_use_memory_management_functions: bool,

    #[yaserde(attribute, rename = "canGetAndSetFMUstate")]
    pub can_get_and_set_fmu_state: bool,

    #[yaserde(attribute, rename = "canSerializeFMUstate")]
    pub can_serialize_fmu_state: bool,

    /// Directional derivatives at communication points
    #[yaserde(attribute, rename = "providesDirectionalDerivative")]
    pub provides_directional_derivative: bool,

    /// List of source file names that are present in the "sources" directory of the FMU and need to be compiled in
    /// order to generate the binary of the FMU (only meaningful for source code FMUs).
    #[yaserde(child, rename = "SourceFiles")]
    pub source_files: SourceFiles,
}
