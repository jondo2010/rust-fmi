// Comprehensive test showing all the features of the procedural macro
use fmi_export::{FmuModel, fmi3::Model};

#[derive(Debug, Default, FmuModel)]
#[model(model_exchange())]
struct ComprehensiveModel {
    /// An output variable showing the main result
    #[variable(causality = Output, start = 42.0, description = "Main output variable")]
    output_value: f64,

    /// A parameter that can be tuned
    #[variable(causality = Parameter, start = 1.0, description = "Tunable parameter")]
    parameter: f64,

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

fn main() {
    println!("=== Comprehensive FMU Model Test ===\n");

    let model = ComprehensiveModel::default();
    println!("Model: {:?}\n", model);

    // Test the new model description functionality
    println!("=== Model Description ===");
    let description = ComprehensiveModel::model_description();
    println!("Model Name: {}", description.model_name);
    println!("FMI Version: {}", description.fmi_version);
    println!("Instantiation Token: {}", description.instantiation_token);
    println!("Description: {:?}", description.description);
    println!("Generation Tool: {:?}", description.generation_tool);
    println!(
        "Generation Date: {:?}\n",
        description.generation_date_and_time
    );

    println!("=== Model Variables ===");
    let vars = ComprehensiveModel::model_variables();

    println!("Float64 variables: {}", vars.float64.len());
    for (i, var) in vars.float64.iter().enumerate() {
        println!(
            "  {}: name='{}', vr={}, causality={:?}, variability={:?}, start={:?}, desc={:?}",
            i,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .name,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .value_reference,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .causality,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            var.start,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .description
        );
    }

    println!("Float32 variables: {}", vars.float32.len());
    for (i, var) in vars.float32.iter().enumerate() {
        println!(
            "  {}: name='{}', vr={}, causality={:?}, variability={:?}, start={:?}, desc={:?}",
            i,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .name,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .value_reference,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .causality,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            var.start,
            var.init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .description
        );
    }

    println!("\n=== Model Structure ===");
    let structure = ComprehensiveModel::model_structure();
    println!("Outputs: {}", structure.outputs.len());
    for (i, output) in structure.outputs.iter().enumerate() {
        println!(
            "  {}: vr={}, dependencies={:?}",
            i, output.value_reference, output.dependencies
        );
    }

    println!(
        "Continuous State Derivatives: {}",
        structure.continuous_state_derivative.len()
    );
    for (i, derivative) in structure.continuous_state_derivative.iter().enumerate() {
        println!(
            "  {}: vr={}, dependencies={:?}, kind={:?}",
            i, derivative.value_reference, derivative.dependencies, derivative.dependencies_kind
        );
    }

    println!("\n=== Value Reference Enum ===");
    println!(
        "OutputValue: {:?} = {}",
        ValueRef::OutputValue,
        ValueRef::OutputValue as u32
    );
    println!(
        "Parameter: {:?} = {}",
        ValueRef::Parameter,
        ValueRef::Parameter as u32
    );
    println!(
        "InputSignal: {:?} = {}",
        ValueRef::InputSignal,
        ValueRef::InputSignal as u32
    );
    println!(
        "InternalState: {:?} = {}",
        ValueRef::InternalState,
        ValueRef::InternalState as u32
    );
    println!(
        "DerInternalState: {:?} = {}",
        ValueRef::DerInternalState,
        ValueRef::DerInternalState as u32
    );
    println!(
        "Float32Output: {:?} = {}",
        ValueRef::Float32Output,
        ValueRef::Float32Output as u32
    );
}
