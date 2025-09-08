use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::traits::FmiInterfaceType;

use super::Annotations;

//TODO: Refactor these structs to use a common base struct / macros

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "InterfaceType")]
pub struct Fmi3InterfaceType {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    /// Short class name according to C syntax, for example, A_B_C.
    #[yaserde(attribute = true, rename = "modelIdentifier")]
    pub model_identifier: String,

    /// If true, a tool is needed to execute the FMU. The FMU implements the communication to this tool.
    #[yaserde(attribute = true, rename = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,

    /// If true, the FMU must be instantiated only once per process.
    #[yaserde(attribute = true, rename = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,

    /// If true, the environment may inquire the internal FMU state and may restore it.
    #[yaserde(attribute = true, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,

    /// If true, the environment may serialize the internal FMU state.
    #[yaserde(attribute = true, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,

    /// If true, the directional derivative of the equations may be retrieved using fmi3GetDirectionalDerivative.
    #[yaserde(attribute = true, rename = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,

    /// If true, the adjoint derivatives of the equations may be retrieved using fmi3GetAdjointDerivative
    #[yaserde(attribute = true, rename = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,

    #[yaserde(attribute = true, rename = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ModelExchange")]
pub struct Fmi3ModelExchange {
    #[yaserde(flatten = true)]
    pub common: Fmi3InterfaceType,

    #[yaserde(attribute = true, rename = "needsCompletedIntegratorStep")]
    pub needs_completed_integrator_step: Option<bool>,

    #[yaserde(attribute = true, rename = "providesEvaluateDiscreteStates")]
    pub provides_evaluate_discrete_states: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "CoSimulation")]
pub struct Fmi3CoSimulation {
    #[yaserde(flatten = true)]
    pub common: Fmi3InterfaceType,

    #[yaserde(attribute = true, rename = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: Option<bool>,

    #[yaserde(attribute = true, rename = "fixedInternalStepSize")]
    pub fixed_internal_step_size: Option<f64>,

    #[yaserde(attribute = true, rename = "maxOutputDerivativeOrder")]
    pub max_output_derivative_order: Option<u32>,

    #[yaserde(attribute = true, rename = "recommendedIntermediateInputSmoothness")]
    pub recommended_intermediate_input_smoothness: Option<i32>,

    #[yaserde(attribute = true, rename = "providesIntermediateUpdate")]
    pub provides_intermediate_update: Option<bool>,

    #[yaserde(attribute = true, rename = "mightReturnEarlyFromDoStep")]
    pub might_return_early_from_do_step: Option<bool>,

    #[yaserde(attribute = true, rename = "canReturnEarlyAfterIntermediateUpdate")]
    pub can_return_early_after_intermediate_update: Option<bool>,

    #[yaserde(attribute = true, rename = "hasEventMode")]
    pub has_event_mode: Option<bool>,

    #[yaserde(attribute = true, rename = "providesEvaluateDiscreteStates")]
    pub provides_evaluate_discrete_states: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ScheduledExecution")]
pub struct Fmi3ScheduledExecution {
    #[yaserde(flatten = true)]
    pub common: Fmi3InterfaceType,
}

macro_rules! impl_fmi_interface_type {
    ($type:ty) => {
        impl FmiInterfaceType for $type {
            fn model_identifier(&self) -> &str {
                &self.common.model_identifier
            }

            fn needs_execution_tool(&self) -> Option<bool> {
                self.common.needs_execution_tool
            }

            fn can_be_instantiated_only_once_per_process(&self) -> Option<bool> {
                self.common.can_be_instantiated_only_once_per_process
            }

            fn can_get_and_set_fmu_state(&self) -> Option<bool> {
                self.common.can_get_and_set_fmu_state
            }

            fn can_serialize_fmu_state(&self) -> Option<bool> {
                self.common.can_serialize_fmu_state
            }

            fn provides_directional_derivatives(&self) -> Option<bool> {
                self.common.provides_directional_derivatives
            }

            fn provides_adjoint_derivatives(&self) -> Option<bool> {
                self.common.provides_adjoint_derivatives
            }

            fn provides_per_element_dependencies(&self) -> Option<bool> {
                self.common.provides_per_element_dependencies
            }
        }
    };
}

impl_fmi_interface_type!(Fmi3ModelExchange);
impl_fmi_interface_type!(Fmi3CoSimulation);
impl_fmi_interface_type!(Fmi3ScheduledExecution);
