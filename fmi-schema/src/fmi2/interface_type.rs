use crate::traits::FmiInterfaceType;

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "File", strict(unknown_attribute, unknown_element))]
pub struct File {
    /// Name of the file including the path relative to the sources directory, using the forward
    /// slash as separator (for example: name = "myFMU.c"; name = "modelExchange/solve.c")
    #[xml(attr = "name")]
    pub name: String,
}

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "SourceFiles", strict(unknown_attribute, unknown_element))]
pub struct SourceFiles {
    #[xml(child = "File")]
    pub files: Vec<File>,
}

/// The FMU includes a model or the communication to a tool that provides a model. The environment
/// provides the simulation engine for the model.
#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ModelExchange", strict(unknown_attribute, unknown_element))]
pub struct ModelExchange {
    /// Short class name according to C-syntax
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,

    /// If true, a tool is needed to execute the model and the FMU just contains the communication
    /// to this tool.
    #[xml(attr = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[xml(attr = "completedIntegratorStepNotNeeded")]
    pub completed_integrator_step_not_needed: Option<bool>,

    #[xml(attr = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[xml(attr = "canNotUseMemoryManagementFunctions")]
    pub can_not_use_memory_management_functions: Option<bool>,

    #[xml(attr = "canGetAndSetFMUstate")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[xml(attr = "canSerializeFMUstate")]
    pub can_serialize_fmu_state: Option<bool>,

    /// If true, the directional derivative of the equations can be computed with
    /// fmi2GetDirectionalDerivative
    #[xml(attr = "providesDirectionalDerivative")]
    pub provides_directional_derivative: Option<bool>,

    /// List of source file names that are present in the "sources" directory of the FMU and need
    /// to be compiled in order to generate the binary of the FMU (only meaningful for source
    /// code FMUs).
    #[xml(child = "SourceFiles")]
    pub source_files: Option<SourceFiles>,
}

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "CoSimulation", strict(unknown_attribute, unknown_element))]
pub struct CoSimulation {
    /// Short class name according to C-syntax
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,

    /// If true, a tool is needed to execute the model and the FMU just contains the communication
    /// to this tool.
    #[xml(attr = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[xml(attr = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: Option<bool>,

    #[xml(attr = "canInterpolateInputs")]
    pub can_interpolate_inputs: Option<bool>,

    #[xml(attr = "maxOutputDerivativeOrder")]
    pub max_output_derivative_order: Option<u32>,

    #[xml(attr = "canRunAsynchronuously")]
    pub can_run_asynchronuously: Option<bool>,

    #[xml(attr = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[xml(attr = "canNotUseMemoryManagementFunctions")]
    pub can_not_use_memory_management_functions: Option<bool>,

    #[xml(attr = "canGetAndSetFMUstate")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[xml(attr = "canSerializeFMUstate")]
    pub can_serialize_fmu_state: Option<bool>,

    /// Directional derivatives at communication points
    #[xml(attr = "providesDirectionalDerivative")]
    pub provides_directional_derivative: Option<bool>,

    /// List of source file names that are present in the "sources" directory of the FMU and need
    /// to be compiled in order to generate the binary of the FMU (only meaningful for source
    /// code FMUs).
    #[xml(child = "SourceFiles")]
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
    use hard_xml::XmlRead;

    use crate::fmi2::ModelExchange;

    #[test]
    fn test_model_exchange() {
        let s = r##"<ModelExchange modelIdentifier="MyLibrary_SpringMassDamper"/>"##;
        let me = ModelExchange::from_str(s).unwrap();
        assert!(me.model_identifier == "MyLibrary_SpringMassDamper");
    }
}
