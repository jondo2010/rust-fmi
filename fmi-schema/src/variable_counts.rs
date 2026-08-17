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

#[cfg(test)]
mod tests {
    use super::Counts;

    #[test]
    fn default_counts_are_zero() {
        let counts = Counts::default();

        assert_eq!(counts.num_constants, 0);
        assert_eq!(counts.num_parameters, 0);
        assert_eq!(counts.num_discrete, 0);
        assert_eq!(counts.num_continuous, 0);
        assert_eq!(counts.num_inputs, 0);
        assert_eq!(counts.num_outputs, 0);
        assert_eq!(counts.num_local, 0);
        assert_eq!(counts.num_independent, 0);
        assert_eq!(counts.num_calculated_parameters, 0);
        assert_eq!(counts.num_real_vars, 0);
        assert_eq!(counts.num_integer_vars, 0);
        assert_eq!(counts.num_enum_vars, 0);
        assert_eq!(counts.num_bool_vars, 0);
        assert_eq!(counts.num_string_vars, 0);
    }

    #[test]
    fn display_labels_every_count() {
        let counts = Counts {
            num_constants: 1,
            num_parameters: 2,
            num_discrete: 3,
            num_continuous: 4,
            num_inputs: 5,
            num_outputs: 6,
            num_local: 7,
            num_independent: 8,
            num_calculated_parameters: 9,
            num_real_vars: 10,
            num_integer_vars: 11,
            num_enum_vars: 12,
            num_bool_vars: 13,
            num_string_vars: 14,
        };

        assert_eq!(
            counts.to_string(),
            "Variable Counts { Constants: 1, Parameters: 2, Discrete: 3, Continuous: 4, Inputs: 5, Outputs: 6, Local: 7, Independent: 8, Calculated parameters: 9, Real: 10, Integer: 11, Enumeration: 12, Boolean: 13, String: 14 }"
        );
    }
}
