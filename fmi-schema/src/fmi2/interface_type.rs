use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::traits::FmiInterfaceType;

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
#[yaserde(tag = "File")]
pub struct File {
    /// Name of the file including the path relative to the sources directory, using the forward
    /// slash as separator (for example: name = "myFMU.c"; name = "modelExchange/solve.c")
    #[yaserde(attribute = true)]
    pub name: String,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
#[yaserde(tag = "SourceFiles")]
pub struct SourceFiles {
    #[yaserde(rename = "File")]
    pub files: Vec<File>,
}

/// The FMU includes a model or the communication to a tool that provides a model. The environment
/// provides the simulation engine for the model.
#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct ModelExchange {
    /// Short class name according to C-syntax
    #[yaserde(attribute = true, rename = "modelIdentifier")]
    pub model_identifier: String,

    /// If true, a tool is needed to execute the model and the FMU just contains the communication
    /// to this tool.
    #[yaserde(attribute = true, rename = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[yaserde(attribute = true, rename = "completedIntegratorStepNotNeeded")]
    pub completed_integrator_step_not_needed: Option<bool>,

    #[yaserde(attribute = true, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[yaserde(attribute = true, rename = "canNotUseMemoryManagementFunctions")]
    pub can_not_use_memory_management_functions: Option<bool>,

    #[yaserde(attribute = true, rename = "canGetAndSetFMUstate")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[yaserde(attribute = true, rename = "canSerializeFMUstate")]
    pub can_serialize_fmu_state: Option<bool>,

    /// If true, the directional derivative of the equations can be computed with
    /// fmi2GetDirectionalDerivative
    #[yaserde(attribute = true, rename = "providesDirectionalDerivative")]
    pub provides_directional_derivative: Option<bool>,

    /// List of source file names that are present in the "sources" directory of the FMU and need
    /// to be compiled in order to generate the binary of the FMU (only meaningful for source
    /// code FMUs).
    #[yaserde(rename = "SourceFiles")]
    pub source_files: Option<SourceFiles>,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct CoSimulation {
    /// Short class name according to C-syntax
    #[yaserde(attribute = true, rename = "modelIdentifier")]
    pub model_identifier: String,

    /// If true, a tool is needed to execute the model and the FMU just contains the communication
    /// to this tool.
    #[yaserde(attribute = true, rename = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[yaserde(attribute = true, rename = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: Option<bool>,

    #[yaserde(attribute = true, rename = "canInterpolateInputs")]
    pub can_interpolate_inputs: Option<bool>,

    #[yaserde(attribute = true, rename = "maxOutputDerivativeOrder")]
    pub max_output_derivative_order: Option<u32>,

    #[yaserde(attribute = true, rename = "canRunAsynchronuously")]
    pub can_run_asynchronuously: Option<bool>,

    #[yaserde(attribute = true, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[yaserde(attribute = true, rename = "canNotUseMemoryManagementFunctions")]
    pub can_not_use_memory_management_functions: Option<bool>,

    #[yaserde(attribute = true, rename = "canGetAndSetFMUstate")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[yaserde(attribute = true, rename = "canSerializeFMUstate")]
    pub can_serialize_fmu_state: Option<bool>,

    /// Directional derivatives at communication points
    #[yaserde(attribute = true, rename = "providesDirectionalDerivative")]
    pub provides_directional_derivative: Option<bool>,

    /// List of source file names that are present in the "sources" directory of the FMU and need
    /// to be compiled in order to generate the binary of the FMU (only meaningful for source
    /// code FMUs).
    #[yaserde(rename = "SourceFiles")]
    pub source_files: Option<SourceFiles>,
}

impl FmiInterfaceType for ModelExchange {
    fn model_identifier(&self) -> &str {
        &self.model_identifier
    }
    fn needs_execution_tool(&self) -> Option<bool> {
        self.needs_execution_tool
    }
    fn can_be_instantiated_only_once_per_process(&self) -> Option<bool> {
        self.can_be_instantiated_only_once_per_process
    }
    fn can_get_and_set_fmu_state(&self) -> Option<bool> {
        self.can_get_and_set_fmu_state
    }
    fn can_serialize_fmu_state(&self) -> Option<bool> {
        self.can_serialize_fmu_state
    }
    fn provides_directional_derivatives(&self) -> Option<bool> {
        self.provides_directional_derivative
    }
    fn provides_adjoint_derivatives(&self) -> Option<bool> {
        None
    }
    fn provides_per_element_dependencies(&self) -> Option<bool> {
        None
    }
}

impl FmiInterfaceType for CoSimulation {
    fn model_identifier(&self) -> &str {
        &self.model_identifier
    }
    fn needs_execution_tool(&self) -> Option<bool> {
        self.needs_execution_tool
    }
    fn can_be_instantiated_only_once_per_process(&self) -> Option<bool> {
        self.can_be_instantiated_only_once_per_process
    }
    fn can_get_and_set_fmu_state(&self) -> Option<bool> {
        self.can_get_and_set_fmu_state
    }
    fn can_serialize_fmu_state(&self) -> Option<bool> {
        self.can_serialize_fmu_state
    }
    fn provides_directional_derivatives(&self) -> Option<bool> {
        self.provides_directional_derivative
    }
    fn provides_adjoint_derivatives(&self) -> Option<bool> {
        None
    }
    fn provides_per_element_dependencies(&self) -> Option<bool> {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::fmi2::ModelExchange;

    #[test]
    fn test_model_exchange() {
        let s = r##"<ModelExchange modelIdentifier="MyLibrary_SpringMassDamper"/>"##;
        let me: ModelExchange = yaserde::de::from_str(s).unwrap();
        assert!(me.model_identifier == "MyLibrary_SpringMassDamper");
    }
}
