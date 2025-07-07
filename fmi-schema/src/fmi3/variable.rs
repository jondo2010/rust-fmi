use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    Float32Attributes, Float64Attributes, Int16Attributes, Int32Attributes, Int64Attributes,
    Int8Attributes, IntegerBaseAttributes, RealBaseAttributes, RealVariableAttributes,
    UInt16Attributes, UInt32Attributes, UInt64Attributes, UInt8Attributes,
};

use crate::default_wrapper;

/// An enumeration that defines the type of a variable.
pub enum VariableType {
    FmiFloat32,
    FmiFloat64,
    FmiInt8,
    FmiUInt8,
    FmiInt16,
    FmiUInt16,
    FmiInt32,
    FmiUInt32,
    FmiInt64,
    FmiUInt64,
    FmiBoolean,
    FmiString,
    FmiBinary,
}

#[cfg(feature = "arrow")]
impl From<VariableType> for arrow::datatypes::DataType {
    fn from(v: VariableType) -> Self {
        match v {
            VariableType::FmiFloat32 => arrow::datatypes::DataType::Float32,
            VariableType::FmiFloat64 => arrow::datatypes::DataType::Float64,
            VariableType::FmiInt8 => arrow::datatypes::DataType::Int8,
            VariableType::FmiUInt8 => arrow::datatypes::DataType::UInt8,
            VariableType::FmiInt16 => arrow::datatypes::DataType::Int16,
            VariableType::FmiUInt16 => arrow::datatypes::DataType::UInt16,
            VariableType::FmiInt32 => arrow::datatypes::DataType::Int32,
            VariableType::FmiUInt32 => arrow::datatypes::DataType::UInt32,
            VariableType::FmiInt64 => arrow::datatypes::DataType::Int64,
            VariableType::FmiUInt64 => arrow::datatypes::DataType::UInt64,
            VariableType::FmiBoolean => arrow::datatypes::DataType::Boolean,
            VariableType::FmiString => arrow::datatypes::DataType::Utf8,
            VariableType::FmiBinary => arrow::datatypes::DataType::Binary,
        }
    }
}

pub trait AbstractVariableTrait {
    /// The full, unique name of the variable.
    fn name(&self) -> &str;
    /// A handle of the variable to efficiently identify the variable value in the model interface
    /// and for references within the modelDescription.xml
    fn value_reference(&self) -> u32;
    /// An optional description string describing the meaning of the variable.
    fn description(&self) -> Option<&str>;
    /// Enumeration that defines the causality of the variable.
    fn causality(&self) -> Causality;
    fn variability(&self) -> Variability;
    fn can_handle_multiple_set_per_time_instant(&self) -> Option<bool>;
    fn data_type(&self) -> VariableType;
}

pub trait ArrayableVariableTrait: AbstractVariableTrait {
    fn dimensions(&self) -> &[Dimension];
    fn intermediate_update(&self) -> Option<bool>;
    fn previous(&self) -> Option<u32>;
}

pub trait TypedArrayableVariableTrait: ArrayableVariableTrait {
    fn declared_type(&self) -> Option<&str>;
}

pub trait InitializableVariableTrait: TypedArrayableVariableTrait {
    fn initial(&self) -> Option<Initial>;
}

macro_rules! impl_abstract_variable {
    ($name:ident, $default_variability:expr) => {
        impl AbstractVariableTrait for $name {
            fn name(&self) -> &str {
                &self
                    .init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .name
            }
            fn value_reference(&self) -> u32 {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .value_reference
            }
            fn description(&self) -> Option<&str> {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .description
                    .as_deref()
            }
            fn causality(&self) -> Causality {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .causality
            }
            fn variability(&self) -> Variability {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .variability
                    .unwrap_or($default_variability)
            }
            fn can_handle_multiple_set_per_time_instant(&self) -> Option<bool> {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .can_handle_multiple_set_per_time_instant
            }
            fn data_type(&self) -> VariableType {
                VariableType::$name
            }
        }
    };
}

macro_rules! impl_arrayable_variable {
    ($name:ident) => {
        impl ArrayableVariableTrait for $name {
            fn dimensions(&self) -> &[Dimension] {
                &self.init_var.typed_arrayable_var.arrayable_var.dimensions
            }
            fn intermediate_update(&self) -> Option<bool> {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .intermediate_update
            }
            fn previous(&self) -> Option<u32> {
                self.init_var.typed_arrayable_var.arrayable_var.previous
            }
        }
    };
}

macro_rules! impl_typed_arrayable_variable {
    ($name:ident) => {
        impl TypedArrayableVariableTrait for $name {
            fn declared_type(&self) -> Option<&str> {
                self.init_var.typed_arrayable_var.declared_type.as_deref()
            }
        }
    };
}

macro_rules! impl_initializable_variable {
    ($name:ident) => {
        impl InitializableVariableTrait for $name {
            fn initial(&self) -> Option<Initial> {
                self.init_var.initial
            }
        }
    };
}

macro_rules! impl_float_type {
    ($name:ident, $root:literal, $type:ty, $float_attr:ident) => {
        #[derive(Default, PartialEq, Debug, YaDeserialize)]
        #[yaserde(rename = $root)]
        pub struct $name {
            #[yaserde(flatten = true)]
            pub base_attr: RealBaseAttributes,
            #[yaserde(flatten = true)]
            pub attr: $float_attr,
            #[yaserde(flatten = true)]
            pub init_var: InitializableVariable,
            #[yaserde(attribute = true, rename = "start")]
            pub start: Vec<$type>,
            #[yaserde(flatten = true)]
            pub real_var_attr: RealVariableAttributes,
        }

        impl_abstract_variable!($name, Variability::Continuous);
        impl_arrayable_variable!($name);
        impl_typed_arrayable_variable!($name);
        impl_initializable_variable!($name);

        impl $name {
            pub fn start(&self) -> &[$type] {
                &self.start
            }

            pub fn derivative(&self) -> Option<u32> {
                self.real_var_attr.derivative
            }

            pub fn reinit(&self) -> Option<bool> {
                self.real_var_attr.reinit
            }
        }
    };
}

macro_rules! impl_integer_type {
    ($name:ident, $root:literal, $type:ty, $int_attr:ident) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        #[yaserde(rename = $root)]
        pub struct $name {
            #[yaserde(flatten = true)]
            pub base_attr: IntegerBaseAttributes,
            #[yaserde(flatten = true)]
            pub int_attr: $int_attr,
            /// Initial or guess value of the variable. During instantiation, the FMU initializes its variables with their start values.
            #[yaserde(attribute = true)]
            pub start: Option<$type>,
            #[yaserde(flatten = true)]
            pub init_var: InitializableVariable,
        }

        impl_abstract_variable!($name, Variability::Discrete);
    };
}

/// Enumeration that defines the causality of the variable.
#[derive(Clone, Copy, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum Causality {
    /// A data value that is constant during the simulation
    #[yaserde(rename = "parameter")]
    Parameter,
    /// A data value that is constant during the simulation and is computed during initialization
    /// or when tunable parameters change.
    #[yaserde(rename = "calculatedParameter")]
    CalculatedParameter,
    /// The variable value can be provided by the importer.
    #[yaserde(rename = "input")]
    Input,
    /// The values of these variables are computed in the FMU and they are designed to be used outside the FMU.
    #[yaserde(rename = "output")]
    Output,
    /// Local variables of the FMU that must not be used for FMU connections
    #[yaserde(rename = "local")]
    #[default]
    Local,
    /// The independent variable (usually time [but could also be, for example, angle]).
    #[yaserde(rename = "independent")]
    Independent,
    #[yaserde(rename = "dependent")]
    Dependent,
    /// The variable value can only be changed in Configuration Mode or Reconfiguration Mode.
    #[yaserde(rename = "structuralParameter")]
    StructuralParameter,
}

/// Enumeration that defines the time dependency of the variable, in other words, it defines the
/// time instants when a variable may be changed by the importer or may change its value due to FMU
/// internal computations, depending on their causality.
///
/// See [https://fmi-standard.org/docs/3.0.1/#variability]
#[derive(Clone, Copy, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum Variability {
    /// The value of the variable never changes.
    #[yaserde(rename = "constant")]
    Constant,
    /// The value of the variable is fixed in super state Initialized, in other words, after
    /// [`exit_initialization_mode()`] was called the variable value does not change anymore. The
    /// default for variables of causality [`Causality::Parameter`],
    /// [`Causality::StructuredParameter`] or [`Causality::CalculatedParameter`] is `Fixed`.
    #[yaserde(rename = "fixed")]
    Fixed,
    /// The value of the variable is constant between events (ME and CS if Event Mode is supported)
    /// and between communication points (CS and SE). A parameter with variability = tunable
    /// may be changed only in Event Mode or, if Event Mode is not supported, at communication
    /// points (CS and SE).
    #[yaserde(rename = "tunable")]
    Tunable,
    /// * Model Exchange: The value of the variable may change only in Event Mode.
    /// * Co-Simulation: If Event Mode is used (see `event_mode_used`), the value of the variable
    ///   may only change in Event Mode. If Event Mode is not used, the value may change at
    ///   communication points and the FMU must detect and handle such events internally. During
    ///   Intermediate Update Mode, discrete variables are not allowed to change.
    /// * Scheduled Execution: The value may change only at communication points.
    #[yaserde(rename = "discrete")]
    #[default]
    Discrete,
    /// Only variables of type [`FmiFloat32`]or [`FmiFloat64`] may be continuous. The default for
    /// variables of type `FmiFloat32` and `FmiFloat64` and causality other than
    /// [`Causality::Parameter`], [`Causality::StructuredParameter`] or
    /// [`Causality::CalculatedParameter`] is continuous. Variables with variability continuous
    /// may change in Initialization Mode and in super state Initialized.
    #[yaserde(rename = "continuous")]
    Continuous,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Dimension {
    /// Defines a constant unsigned 64-bit integer size for this dimension. The variability of the
    /// dimension size is constant in this case.
    #[yaserde(attribute = true)]
    pub start: Option<u64>,
    /// If the present, it defines the size of this dimension to be the value of the variable with
    /// the value reference given by the `value_reference` attribute. The referenced variable
    /// must be a variable of type `UInt64`, and must either be a constant (i.e. with
    /// variability = constant) or a structural parameter (i.e. with causality =
    /// structuralParameter). The variability of the dimension size is in this case the variability
    /// of the referenced variable. A structural parameter must be a variable of type `UInt64`
    /// only if it is referenced in `Dimension`.
    #[yaserde(attribute = true, rename = "valueReference")]
    pub value_reference: Option<u32>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct AbstractVariable {
    #[yaserde(attribute = true)]
    pub name: String,
    #[yaserde(attribute = true, rename = "valueReference")]
    pub value_reference: u32,
    #[yaserde(attribute = true)]
    pub description: Option<String>,
    #[yaserde(attribute = true, default = "default_wrapper")]
    pub causality: Causality,
    #[yaserde(attribute = true)]
    pub variability: Option<Variability>,
    #[yaserde(attribute = true, rename = "canHandleMultipleSetPerTimeInstant")]
    pub can_handle_multiple_set_per_time_instant: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct ArrayableVariable {
    #[yaserde(flatten = true)]
    pub abstract_var: AbstractVariable,
    /// Each `Dimension` element specifies the size of one dimension of the array
    #[yaserde(rename = "Dimension")]
    pub dimensions: Vec<Dimension>,
    #[yaserde(attribute = true, rename = "intermediateUpdate")]
    pub intermediate_update: Option<bool>,
    #[yaserde(attribute = true, rename = "previous")]
    pub previous: Option<u32>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct TypedArrayableVariable {
    #[yaserde(flatten = true)]
    pub arrayable_var: ArrayableVariable,
    #[yaserde(attribute = true, rename = "declaredType")]
    pub declared_type: Option<String>,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum Initial {
    #[yaserde(rename = "exact")]
    #[default]
    Exact,
    #[yaserde(rename = "approx")]
    Approx,
    #[yaserde(rename = "calculated")]
    Calculated,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct InitializableVariable {
    #[yaserde(flatten = true)]
    pub typed_arrayable_var: TypedArrayableVariable,
    #[yaserde(attribute = true)]
    pub initial: Option<Initial>,
}

impl_float_type!(FmiFloat32, "Float32", f32, Float32Attributes);
impl_float_type!(FmiFloat64, "Float64", f64, Float64Attributes);

impl_integer_type!(FmiInt8, "Int8", i8, Int8Attributes);
impl_integer_type!(FmiUInt8, "UInt8", u8, UInt8Attributes);
impl_integer_type!(FmiInt16, "Int16", i16, Int16Attributes);
impl_integer_type!(FmiUInt16, "UInt16", u16, UInt16Attributes);
impl_integer_type!(FmiInt32, "Int32", i32, Int32Attributes);
impl_integer_type!(FmiUInt32, "UInt32", u32, UInt32Attributes);
impl_integer_type!(FmiInt64, "Int64", i64, Int64Attributes);
impl_integer_type!(FmiUInt64, "UInt64", u64, UInt64Attributes);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiBoolean {
    #[yaserde(attribute = true, flatten = true)]
    pub start: Vec<bool>,
    #[yaserde(flatten = true)]
    pub init_var: InitializableVariable,
}

impl_abstract_variable!(FmiBoolean, Variability::Discrete);
impl_arrayable_variable!(FmiBoolean);
impl_typed_arrayable_variable!(FmiBoolean);
impl_initializable_variable!(FmiBoolean);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct StringStart {
    #[yaserde(attribute = true, rename = "value")]
    pub value: String,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiString {
    #[yaserde(rename = "Start")]
    pub start: Vec<StringStart>,
    #[yaserde(flatten = true)]
    pub init_var: InitializableVariable,
}

impl FmiString {
    pub fn start(&self) -> impl Iterator<Item = &str> {
        self.start.iter().map(|s| s.value.as_str())
    }
}

impl_abstract_variable!(FmiString, Variability::Discrete);
impl_arrayable_variable!(FmiString);
impl_typed_arrayable_variable!(FmiString);
impl_initializable_variable!(FmiString);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct BinaryStart {
    #[yaserde(attribute = true, rename = "value")]
    pub value: String,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiBinary {
    #[yaserde(attribute = true, rename = "start")]
    pub start: Vec<BinaryStart>,
    #[yaserde(attribute = true, default = "default_mime_type")]
    pub mime_type: String,
    #[yaserde(attribute = true)]
    pub max_size: Option<u32>,
    #[yaserde(flatten = true)]
    pub init_var: InitializableVariable,
}

fn default_mime_type() -> String {
    "application/octet-stream".into()
}

impl_abstract_variable!(FmiBinary, Variability::Discrete);

// #[derive(Debug, YaSerialize, YaDeserialize)]
// #[yaserde(root = "ModelVariables")]
// pub enum Fmi3Variable {
// #[yaserde(flatten, rename = "Float32")]
// Float32(FmiFloat32),
// #[yaserde(flatten, rename = "Float64")]
// Float64(FmiFloat64),
// }
//
// impl Default for Fmi3Variable {
// fn default() -> Self {
// Fmi3Variable::Float32(FmiFloat32::default())
// }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int16() {
        let xml = r#"<Int16 name="Int16_input" valueReference="15" causality="input" start="0"/>"#;
        let var: FmiInt16 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "Int16_input");
        assert_eq!(var.value_reference(), 15);
        assert_eq!(var.causality(), Causality::Input);
        assert_eq!(var.start, Some(0));
        assert_eq!(var.variability(), Variability::Discrete); // The default for non-float types should be discrete
    }

    #[test]
    fn test_float64() {
        let xml = r#"<Float64
        name="g"
        valueReference="5"
        causality="parameter"
        variability="fixed"
        initial="exact"
        declaredType="Acceleration"
        start="-9.81"
        derivative="1"
        description="Gravity acting on the ball"
    />"#;
        let var: FmiFloat64 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "g");
        assert_eq!(var.value_reference(), 5);
        assert_eq!(var.variability(), Variability::Fixed);
        assert_eq!(var.initial(), Some(Initial::Exact));
        assert_eq!(var.causality(), Causality::Parameter);
        assert_eq!(var.declared_type(), Some("Acceleration"));
        assert_eq!(var.start(), &[-9.81]);
        assert_eq!(var.derivative(), Some(1));
        assert_eq!(var.description(), Some("Gravity acting on the ball"));
        assert_eq!(var.can_handle_multiple_set_per_time_instant(), None);
        assert_eq!(var.intermediate_update(), None);
    }

    #[test]
    fn test_dim_f64() {
        let xml = r#"<Float64
        name="A"
        valueReference="4"
        description="Matrix coefficient A"
        causality="parameter"
        variability="tunable"
        start="1 0 0 0 1 0 0 0 1">
        <Dimension valueReference="2"/>
        <Dimension valueReference="2"/>
        </Float64>"#;

        let var: FmiFloat64 = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "A");
        assert_eq!(var.value_reference(), 4);
        assert_eq!(var.variability(), Variability::Tunable);
        assert_eq!(var.causality(), Causality::Parameter);
        assert_eq!(var.description(), Some("Matrix coefficient A"));
        assert_eq!(var.start, vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
        assert_eq!(var.dimensions().len(), 2);
        assert_eq!(var.dimensions()[0].value_reference, Some(2));
    }

    #[test]
    fn test_string() {
        let xml = r#"<String name="String_parameter" valueReference="29" causality="parameter" variability="fixed">
        <Start value="Set me!"/>
    </String>"#;

        let var: FmiString = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "String_parameter");
        assert_eq!(var.value_reference(), 29);
        assert_eq!(var.variability(), Variability::Fixed);
        assert_eq!(var.causality(), Causality::Parameter);
        assert_eq!(var.start().next().unwrap(), "Set me!");
    }
}
