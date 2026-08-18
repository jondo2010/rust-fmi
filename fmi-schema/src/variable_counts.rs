use std::fmt::Display;

/// Collects counts of variables in the model
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

impl Display for Counts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Variable Counts")
            .field("Constants", &self.num_constants)
            .field("Parameters", &self.num_parameters)
            .field("Discrete", &self.num_discrete)
            .field("Continuous", &self.num_continuous)
            .field("Inputs", &self.num_inputs)
            .field("Outputs", &self.num_outputs)
            .field("Local", &self.num_local)
            .field("Independent", &self.num_independent)
            .field("Calculated parameters", &self.num_calculated_parameters)
            .field("Real", &self.num_real_vars)
            .field("Integer", &self.num_integer_vars)
            .field("Enumeration", &self.num_enum_vars)
            .field("Boolean", &self.num_bool_vars)
            .field("String", &self.num_string_vars)
            .finish()
    }
}

pub trait VariableCounts {
    fn model_counts(&self) -> Counts;
}
