#![deny(clippy::all)]
//! Example port of the Feedthrough FMU from the Reference FMUs

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{DefaultLoggingCategory, ModelContext, UserModel},
    FmuModel,
};

#[derive(FmuModel, Default, Debug)]
struct Feedthrough {
    #[variable(causality = Input, start = 0.0)]
    float32_continuous_input: f32,
    #[variable(causality = Output)]
    float32_continuous_output: f32,
    #[variable(causality = Input, start = 0.0)]
    float32_discrete_input: f32,
    #[variable(causality = Output)]
    float32_discrete_output: f32,

    #[variable(causality = Parameter, variability = Fixed, start = 0.0)]
    float64_fixed_parameter: f64,
    #[variable(causality = Parameter, variability = Tunable, start = 0.0)]
    float64_tunable_parameter: f64,
    #[variable(causality = Input, start = 0.0)]
    float64_continuous_input: f64,
    #[variable(causality = Output, initial = Calculated)]
    float64_continuous_output: f64,
    #[variable(causality = Input, variability = Discrete, start = 0.0)]
    float64_discrete_input: f64,
    #[variable(causality = Output, variability = Discrete, initial = Calculated)]
    float64_discrete_output: f64,

    #[variable(causality = Input, start = 0)]
    int8_input: i8,
    #[variable(causality = Output)]
    int8_output: i8,

    #[variable(causality = Input, start = 0)]
    uint8_input: u8,
    #[variable(causality = Output)]
    uint8_output: u8,

    #[variable(causality = Input, start = 0)]
    int16_input: i16,
    #[variable(causality = Output)]
    int16_output: i16,

    #[variable(causality = Input, start = 0)]
    uint16_input: u16,
    #[variable(causality = Output)]
    uint16_output: u16,

    #[variable(causality = Input, start = 0)]
    int32_input: i32,
    #[variable(causality = Output)]
    int32_output: i32,

    #[variable(causality = Input, start = 0)]
    uint32_input: u32,
    #[variable(causality = Output)]
    uint32_output: u32,

    #[variable(causality = Input, start = 0)]
    int64_input: i64,
    #[variable(causality = Output)]
    int64_output: i64,

    #[variable(causality = Input, start = 0)]
    uint64_input: u64,
    #[variable(causality = Output)]
    uint64_output: u64,

    #[variable(causality = Input, start = false)]
    boolean_input: bool,
    #[variable(causality = Output, initial = Calculated)]
    boolean_output: bool,

    #[variable(causality = Input, start = "Set me!")]
    string_input: String,
    #[variable(causality = Output)]
    string_output: String,

    #[variable(causality = Input, start = b"foo")]
    binary_input: bytes::BytesMut,
    #[variable(causality = Output)]
    binary_output: bytes::BytesMut,
}

impl UserModel for Feedthrough {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Result<Fmi3Res, Fmi3Error> {
        self.float32_continuous_output = self.float32_continuous_input;
        self.float32_discrete_output = self.float32_discrete_input;

        self.float64_continuous_output = self.float64_continuous_input;
        self.float64_discrete_output = self.float64_discrete_input;

        self.int8_output = self.int8_input;
        self.uint8_output = self.uint8_input;
        self.int16_output = self.int16_input;
        self.uint16_output = self.uint16_input;
        self.int32_output = self.int32_input;
        self.uint32_output = self.uint32_input;
        self.int64_output = self.int64_input;
        self.uint64_output = self.uint64_input;
        self.boolean_output = self.boolean_input;

        self.string_output = self.string_input.clone();

        self.binary_output.copy_from_slice(&self.binary_input);

        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(Feedthrough);
