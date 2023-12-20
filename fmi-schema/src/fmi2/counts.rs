use super::{Causality, FmiModelDescription, ScalarVariableElement, Variability};

#[derive(Debug, Default)]
pub struct Counts {
    pub num_constants: usize,
    pub num_parameters: usize,
    pub num_discrete: usize,
    pub num_continuous: usize,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub num_local: usize,
    pub num_independent: usize,
    pub num_calculated_parameters: usize,
    pub num_real_vars: usize,
    pub num_integer_vars: usize,
    pub num_enum_vars: usize,
    pub num_bool_vars: usize,
    pub num_string_vars: usize,
}

impl FmiModelDescription {
    /// Collect counts of variables in the model
    pub fn model_counts(&self) -> Counts {
        self.model_variables
            .variables
            .iter()
            .fold(Counts::default(), |mut cts, ref sv| {
                match sv.variability {
                    Variability::Constant => {
                        cts.num_constants += 1;
                    }
                    Variability::Continuous => {
                        cts.num_continuous += 1;
                    }
                    Variability::Discrete => {
                        cts.num_discrete += 1;
                    }
                    _ => {}
                }
                match sv.causality {
                    Causality::CalculatedParameter => {
                        cts.num_calculated_parameters += 1;
                    }
                    Causality::Parameter => {
                        cts.num_parameters += 1;
                    }
                    Causality::Input => {
                        cts.num_inputs += 1;
                    }
                    Causality::Output => {
                        cts.num_outputs += 1;
                    }
                    Causality::Local => {
                        cts.num_local += 1;
                    }
                    Causality::Independent => {
                        cts.num_independent += 1;
                    }
                    _ => {}
                }
                match sv.elem {
                    ScalarVariableElement::Real { .. } => {
                        cts.num_real_vars += 1;
                    }
                    ScalarVariableElement::Integer { .. } => {
                        cts.num_integer_vars += 1;
                    }
                    ScalarVariableElement::Enumeration { .. } => {
                        cts.num_enum_vars += 1;
                    }
                    ScalarVariableElement::Boolean { .. } => {
                        cts.num_bool_vars += 1;
                    }
                    ScalarVariableElement::String { .. } => {
                        cts.num_string_vars += 1;
                    }
                }
                cts
            })
    }
}
