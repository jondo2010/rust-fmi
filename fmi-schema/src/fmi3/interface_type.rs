use yaserde_derive::{YaDeserialize, YaSerialize};

use super::Annotations;

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "InterfaceType")]
pub struct Fmi3InterfaceType {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,

    #[yaserde(attribute, rename = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[yaserde(attribute, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[yaserde(attribute, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ModelExchange")]
pub struct Fmi3ModelExchange {
    #[yaserde(attribute, rename = "needsCompletedIntegratorStep")]
    pub needs_completed_integrator_step: Option<bool>,

    #[yaserde(attribute, rename = "providesEvaluateDiscreteStates")]
    pub provides_evaluate_discrete_states: Option<bool>,

    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,

    #[yaserde(attribute, rename = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[yaserde(attribute, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[yaserde(attribute, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "CoSimulation")]
pub struct Fmi3CoSimulation {
    #[yaserde(attribute, rename = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: Option<bool>,

    #[yaserde(attribute, rename = "fixedInternalStepSize")]
    pub fixed_internal_step_size: Option<f64>,

    #[yaserde(attribute, rename = "maxOutputDerivativeOrder")]
    pub max_output_derivative_order: Option<u32>,

    #[yaserde(attribute, rename = "recommendedIntermediateInputSmoothness")]
    pub recommended_intermediate_input_smoothness: Option<i32>,

    #[yaserde(attribute, rename = "providesIntermediateUpdate")]
    pub provides_intermediate_update: Option<bool>,

    #[yaserde(attribute, rename = "mightReturnEarlyFromDoStep")]
    pub might_return_early_from_do_step: Option<bool>,

    #[yaserde(attribute, rename = "canReturnEarlyAfterIntermediateUpdate")]
    pub can_return_early_after_intermediate_update: Option<bool>,

    #[yaserde(attribute, rename = "hasEventMode")]
    pub has_event_mode: Option<bool>,

    #[yaserde(attribute, rename = "providesEvaluateDiscreteStates")]
    pub provides_evaluate_discrete_states: Option<bool>,

    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,

    #[yaserde(attribute, rename = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[yaserde(attribute, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[yaserde(attribute, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ScheduledExecution")]
pub struct Fmi3ScheduledExecution {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,

    #[yaserde(attribute, rename = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    #[yaserde(attribute, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    #[yaserde(attribute, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,

    #[yaserde(attribute, rename = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,

    #[yaserde(attribute, rename = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
}
