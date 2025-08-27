// Comprehensive test showing all the features of the procedural macro
use fmi_export::{FmuModel, fmi3::Model};

#[derive(Debug, Default, FmuModel)]
#[model(model_exchange())]
struct ComprehensiveModel {
    /// An output variable showing the main result
    #[variable(causality = Output, start = 42.0, description = "Main output variable")]
    output_value: f64,

    /// A parameter that can be tuned
    #[variable(causality = Parameter, start = false, description = "Tunable parameter")]
    parameter: bool,

    /// An input to the model
    #[variable(causality = Input, start = 0.0, description = "External input")]
    input_signal: f64,

    /// A continuous state variable
    #[variable(causality = Local, state = true, start = 2.0, description = "Internal state")]
    internal_state: f64,

    /// A 32-bit float for demonstration
    #[variable(causality = Output, start = 3.14, description = "32-bit output")]
    float32_output: f32,
}
