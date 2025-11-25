use crate::traits::FmiInterfaceType;

use super::Annotations;

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ModelExchange", strict(unknown_attribute, unknown_element))]
pub struct Fmi3ModelExchange {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    /// Short class name according to C syntax, for example, A_B_C.
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,
    /// If true, a tool is needed to execute the FMU. The FMU implements the communication to this tool.
    #[xml(attr = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,
    /// If true, the FMU must be instantiated only once per process.
    #[xml(attr = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,
    /// If true, the environment may inquire the internal FMU state and may restore it.
    #[xml(attr = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,
    /// If true, the environment may serialize the internal FMU state.
    #[xml(attr = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,
    /// If true, the directional derivative of the equations may be retrieved using fmi3GetDirectionalDerivative.
    #[xml(attr = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,
    /// If true, the adjoint derivatives of the equations may be retrieved using fmi3GetAdjointDerivative
    #[xml(attr = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,
    #[xml(attr = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
    #[xml(attr = "needsCompletedIntegratorStep")]
    pub needs_completed_integrator_step: Option<bool>,
    #[xml(attr = "providesEvaluateDiscreteStates")]
    pub provides_evaluate_discrete_states: Option<bool>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "CoSimulation", strict(unknown_attribute, unknown_element))]
pub struct Fmi3CoSimulation {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    /// Short class name according to C syntax, for example, A_B_C.
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,
    /// If true, a tool is needed to execute the FMU. The FMU implements the communication to this tool.
    #[xml(attr = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,
    /// If true, the FMU must be instantiated only once per process.
    #[xml(attr = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,
    /// If true, the environment may inquire the internal FMU state and may restore it.
    #[xml(attr = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,
    /// If true, the environment may serialize the internal FMU state.
    #[xml(attr = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,
    /// If true, the directional derivative of the equations may be retrieved using fmi3GetDirectionalDerivative.
    #[xml(attr = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,
    /// If true, the adjoint derivatives of the equations may be retrieved using fmi3GetAdjointDerivative
    #[xml(attr = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,
    #[xml(attr = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
    #[xml(attr = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: Option<bool>,
    #[xml(attr = "fixedInternalStepSize")]
    pub fixed_internal_step_size: Option<f64>,
    #[xml(attr = "maxOutputDerivativeOrder")]
    pub max_output_derivative_order: Option<u32>,
    #[xml(attr = "recommendedIntermediateInputSmoothness")]
    pub recommended_intermediate_input_smoothness: Option<i32>,
    #[xml(attr = "providesIntermediateUpdate")]
    pub provides_intermediate_update: Option<bool>,
    #[xml(attr = "mightReturnEarlyFromDoStep")]
    pub might_return_early_from_do_step: Option<bool>,
    #[xml(attr = "canReturnEarlyAfterIntermediateUpdate")]
    pub can_return_early_after_intermediate_update: Option<bool>,
    #[xml(attr = "hasEventMode")]
    pub has_event_mode: Option<bool>,
    #[xml(attr = "providesEvaluateDiscreteStates")]
    pub provides_evaluate_discrete_states: Option<bool>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ScheduledExecution", strict(unknown_attribute, unknown_element))]
pub struct Fmi3ScheduledExecution {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    /// Short class name according to C syntax, for example, A_B_C.
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,
    /// If true, a tool is needed to execute the FMU. The FMU implements the communication to this tool.
    #[xml(attr = "needsExecutionTool")]
    pub needs_execution_tool: Option<bool>,
    /// If true, the FMU must be instantiated only once per process.
    #[xml(attr = "canBeInstantiatedOnlyOncePerProcess")]
    pub can_be_instantiated_only_once_per_process: Option<bool>,
    /// If true, the environment may inquire the internal FMU state and may restore it.
    #[xml(attr = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: Option<bool>,
    /// If true, the environment may serialize the internal FMU state.
    #[xml(attr = "canSerializeFMUState")]
    pub can_serialize_fmu_state: Option<bool>,
    /// If true, the directional derivative of the equations may be retrieved using fmi3GetDirectionalDerivative.
    #[xml(attr = "providesDirectionalDerivatives")]
    pub provides_directional_derivatives: Option<bool>,
    /// If true, the adjoint derivatives of the equations may be retrieved using fmi3GetAdjointDerivative
    #[xml(attr = "providesAdjointDerivatives")]
    pub provides_adjoint_derivatives: Option<bool>,
    #[xml(attr = "providesPerElementDependencies")]
    pub provides_per_element_dependencies: Option<bool>,
}

/// Macro to generate FMI3 interface type structs with common fields from fmi3InterfaceType
macro_rules! impl_fmi_interface_type {
    ($name:ident) => {
        impl FmiInterfaceType for $name {
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
                self.provides_directional_derivatives
            }

            fn provides_adjoint_derivatives(&self) -> Option<bool> {
                self.provides_adjoint_derivatives
            }

            fn provides_per_element_dependencies(&self) -> Option<bool> {
                self.provides_per_element_dependencies
            }
        }
    };
}

impl_fmi_interface_type!(Fmi3ModelExchange);
impl_fmi_interface_type!(Fmi3CoSimulation);
impl_fmi_interface_type!(Fmi3ScheduledExecution);

#[cfg(test)]
mod tests {
    use super::*;
    use hard_xml::{XmlRead, XmlWrite};

    #[test]
    fn test_model_exchange_roundtrip() {
        let xml = r#"<ModelExchange modelIdentifier="test" needsCompletedIntegratorStep="true" providesEvaluateDiscreteStates="false"/>"#;

        let me = Fmi3ModelExchange::from_str(xml).unwrap();
        assert_eq!(me.model_identifier, "test");
        assert_eq!(me.needs_completed_integrator_step, Some(true));
        assert_eq!(me.provides_evaluate_discrete_states, Some(false));

        let xml_out = me.to_string().unwrap();
        let me2 = Fmi3ModelExchange::from_str(&xml_out).unwrap();
        assert_eq!(me, me2);
    }

    #[test]
    fn test_co_simulation_attributes() {
        let xml = r#"<CoSimulation modelIdentifier="test" canHandleVariableCommunicationStepSize="true" hasEventMode="false"/>"#;

        let cs = Fmi3CoSimulation::from_str(xml).unwrap();
        assert_eq!(cs.model_identifier, "test");
        assert_eq!(cs.can_handle_variable_communication_step_size, Some(true));
        assert_eq!(cs.has_event_mode, Some(false));
    }

    #[test]
    fn test_scheduled_execution_basic() {
        let xml = r#"<ScheduledExecution modelIdentifier="test"/>"#;

        let se = Fmi3ScheduledExecution::from_str(xml).unwrap();
        assert_eq!(se.model_identifier, "test");
    }

    #[test]
    fn test_fmi_interface_type_trait() {
        let me = Fmi3ModelExchange {
            model_identifier: "test".to_string(),
            needs_execution_tool: Some(true),
            can_get_and_set_fmu_state: Some(false),
            ..Default::default()
        };

        assert_eq!(me.model_identifier(), "test");
        assert_eq!(me.needs_execution_tool(), Some(true));
        assert_eq!(me.can_get_and_set_fmu_state(), Some(false));
    }
}
