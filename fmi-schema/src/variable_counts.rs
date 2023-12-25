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

pub trait VariableCounts {
    fn model_counts(&self) -> Counts;
}
