use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    Annotations, Float32Attributes, Float64Attributes, Int8Attributes, Int16Attributes,
    Int32Attributes, Int64Attributes, IntegerBaseAttributes, RealBaseAttributes,
    RealVariableAttributes, UInt8Attributes, UInt16Attributes, UInt32Attributes, UInt64Attributes,
};

use crate::{Error, default_wrapper};

use std::fmt::Debug;

mod dimension;
mod model_variables;
#[cfg(test)]
mod tests;

pub use dimension::Dimension;
pub use model_variables::{AppendToModelVariables, ModelVariables};

/// An enumeration that defines the type of a variable.
#[derive(Debug, PartialEq, Clone, Copy)]
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
    fn annotations(&self) -> Option<&Annotations>;
}

pub trait ArrayableVariableTrait: AbstractVariableTrait {
    /// Each `Dimension` element specifies the size of one dimension of the array
    fn dimensions(&self) -> &[Dimension];
    /// Extend the dimensions of the variable
    fn add_dimensions(&mut self, dims: &[Dimension]);
    /// If `true`, the variable can be updated during intermediate update mode.
    fn intermediate_update(&self) -> Option<bool>;
    /// The value reference of the variable that provides the previous value of this variable.
    fn previous(&self) -> Option<u32>;
}

pub trait TypedArrayableVariableTrait: ArrayableVariableTrait {
    fn declared_type(&self) -> Option<&str>;
}

pub trait InitializableVariableTrait<T>: TypedArrayableVariableTrait {
    fn initial(&self) -> Option<Initial>;

    fn start(&self) -> Option<&[T]>;
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
            fn annotations(&self) -> Option<&Annotations> {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .abstract_var
                    .annotations
                    .as_ref()
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
            fn add_dimensions(&mut self, dims: &[Dimension]) {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .dimensions
                    .extend_from_slice(dims);
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
    ($name:ident, $type:ty) => {
        impl InitializableVariableTrait<$type> for $name {
            fn initial(&self) -> Option<Initial> {
                self.init_var.initial
            }

            fn start(&self) -> Option<&[$type]> {
                self.start.as_deref()
            }
        }
    };
}

macro_rules! impl_float_type {
    ($name:ident, $root:literal, $type:ty, $float_attr:ident) => {
        #[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
        #[yaserde(rename = $root)]
        pub struct $name {
            #[yaserde(flatten = true)]
            pub base_attr: RealBaseAttributes,
            #[yaserde(flatten = true)]
            pub attr: $float_attr,
            #[yaserde(flatten = true)]
            pub init_var: InitializableVariable,
            #[yaserde(attribute = true, rename = "start")]
            pub start: Option<Vec<$type>>,
            #[yaserde(flatten = true)]
            pub real_var_attr: RealVariableAttributes,
        }

        impl_abstract_variable!($name, Variability::Continuous);
        impl_arrayable_variable!($name);
        impl_typed_arrayable_variable!($name);
        impl_initializable_variable!($name, $type);

        impl $name {
            pub fn derivative(&self) -> Option<u32> {
                self.real_var_attr.derivative
            }

            pub fn reinit(&self) -> Option<bool> {
                self.real_var_attr.reinit
            }

            /// Create a new FMI variable with the given parameters
            pub fn new(
                name: String,
                value_reference: u32,
                description: Option<String>,
                causality: Causality,
                variability: Variability,
                start: Option<Vec<$type>>,
                initial: Option<Initial>,
            ) -> Self {
                Self {
                    start,
                    init_var: InitializableVariable {
                        typed_arrayable_var: TypedArrayableVariable {
                            arrayable_var: ArrayableVariable {
                                abstract_var: AbstractVariable {
                                    name,
                                    value_reference,
                                    description,
                                    causality,
                                    variability: Some(variability),
                                    can_handle_multiple_set_per_time_instant: None,
                                    annotations: None,
                                },
                                dimensions: vec![],
                                intermediate_update: None,
                                previous: None,
                            },
                            declared_type: None,
                        },
                        initial,
                    },
                    ..Default::default()
                }
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
            #[yaserde(attribute = true, flatten = true)]
            pub start: Option<Vec<$type>>,
            #[yaserde(flatten = true)]
            pub init_var: InitializableVariable,
        }

        impl_abstract_variable!($name, Variability::Discrete);
        impl_arrayable_variable!($name);
        impl_typed_arrayable_variable!($name);
        impl_initializable_variable!($name, $type);

        impl $name {
            /// Create a new FMI integer variable with the given parameters
            pub fn new(
                name: String,
                value_reference: u32,
                description: Option<String>,
                causality: Causality,
                variability: Variability,
                start: Option<Vec<$type>>,
                initial: Option<Initial>,
            ) -> Self {
                Self {
                    start,
                    init_var: InitializableVariable {
                        typed_arrayable_var: TypedArrayableVariable {
                            arrayable_var: ArrayableVariable {
                                abstract_var: AbstractVariable {
                                    name,
                                    value_reference,
                                    description,
                                    causality,
                                    variability: Some(variability),
                                    can_handle_multiple_set_per_time_instant: None,
                                    annotations: None,
                                },
                                dimensions: vec![],
                                intermediate_update: None,
                                previous: None,
                            },
                            declared_type: None,
                        },
                        initial,
                    },
                    ..Default::default()
                }
            }
        }
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
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
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
    pub start: Option<Vec<bool>>,
    #[yaserde(flatten = true)]
    pub init_var: InitializableVariable,
}

impl_abstract_variable!(FmiBoolean, Variability::Discrete);
impl_arrayable_variable!(FmiBoolean);
impl_typed_arrayable_variable!(FmiBoolean);
impl_initializable_variable!(FmiBoolean, bool);

impl FmiBoolean {
    /// Create a new FMI boolean variable with the given parameters
    pub fn new(
        name: String,
        value_reference: u32,
        description: Option<String>,
        causality: Causality,
        variability: Variability,
        start: Option<Vec<bool>>,
        initial: Option<Initial>,
    ) -> Self {
        Self {
            start,
            init_var: InitializableVariable {
                typed_arrayable_var: TypedArrayableVariable {
                    arrayable_var: ArrayableVariable {
                        abstract_var: AbstractVariable {
                            name,
                            value_reference,
                            description,
                            causality,
                            variability: Some(variability),
                            can_handle_multiple_set_per_time_instant: None,
                            annotations: None,
                        },
                        dimensions: vec![],
                        intermediate_update: None,
                        previous: None,
                    },
                    declared_type: None,
                },
                initial,
            },
        }
    }
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct StringStart {
    #[yaserde(attribute = true, rename = "value")]
    pub value: String,
}

#[derive(Default, PartialEq, Debug)]
pub struct FmiString {
    pub start: Option<Vec<StringStart>>,
    pub init_var: InitializableVariable,
}

impl FmiString {
    /// Create a new FMI string variable with the given parameters
    pub fn new(
        name: String,
        value_reference: u32,
        description: Option<String>,
        causality: Causality,
        variability: Variability,
        start: Option<Vec<String>>,
        initial: Option<Initial>,
    ) -> Self {
        Self {
            start: start.map(|s| s.into_iter().map(|value| StringStart { value }).collect()),
            init_var: InitializableVariable {
                typed_arrayable_var: TypedArrayableVariable {
                    arrayable_var: ArrayableVariable {
                        abstract_var: AbstractVariable {
                            name,
                            value_reference,
                            description,
                            causality,
                            variability: Some(variability),
                            can_handle_multiple_set_per_time_instant: None,
                            annotations: None,
                        },
                        dimensions: vec![],
                        intermediate_update: None,
                        previous: None,
                    },
                    declared_type: None,
                },
                initial,
            },
        }
    }
}

impl_abstract_variable!(FmiString, Variability::Discrete);
impl_arrayable_variable!(FmiString);
impl_typed_arrayable_variable!(FmiString);
impl_initializable_variable!(FmiString, StringStart);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct BinaryStart {
    #[yaserde(attribute = true, rename = "value")]
    pub value: String,
}

impl BinaryStart {
    /// Parse the hex string into a byte vector.
    pub fn as_bytes(&self) -> Result<Vec<u8>, Error> {
        let raw: &str = self.value.as_ref();
        let s = raw
            .strip_prefix("0x")
            .or_else(|| raw.strip_prefix("0X"))
            .unwrap_or(raw);
        let s: String = s
            .chars()
            .filter(|c| !c.is_ascii_whitespace() && *c != '_')
            .collect();
        assert!(
            s.len() % 2 == 0,
            "hex string must have an even number of digits"
        );
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|e| Error::Model(format!("failed to parse hex string: {}", e)))
    }
}

#[derive(PartialEq, Debug)]
pub struct FmiBinary {
    pub start: Option<Vec<BinaryStart>>,
    pub mime_type: String,
    pub max_size: Option<u32>,
    pub init_var: InitializableVariable,
}

impl Default for FmiBinary {
    fn default() -> Self {
        Self {
            start: None,
            mime_type: default_mime_type(),
            max_size: None,
            init_var: InitializableVariable::default(),
        }
    }
}

fn default_mime_type() -> String {
    "application/octet-stream".into()
}

impl FmiBinary {
    /// Create a new FMI binary variable with the given parameters
    pub fn new(
        name: String,
        value_reference: u32,
        description: Option<String>,
        causality: Causality,
        variability: Variability,
        start: Option<Vec<String>>,
        initial: Option<Initial>,
    ) -> Self {
        Self {
            start: start.map(|s| s.into_iter().map(|value| BinaryStart { value }).collect()),
            mime_type: default_mime_type(),
            max_size: None,
            init_var: InitializableVariable {
                typed_arrayable_var: TypedArrayableVariable {
                    arrayable_var: ArrayableVariable {
                        abstract_var: AbstractVariable {
                            name,
                            value_reference,
                            description,
                            causality,
                            variability: Some(variability),
                            can_handle_multiple_set_per_time_instant: None,
                            annotations: None,
                        },
                        dimensions: vec![],
                        intermediate_update: None,
                        previous: None,
                    },
                    declared_type: None,
                },
                initial,
            },
        }
    }
}

impl_abstract_variable!(FmiBinary, Variability::Discrete);
impl_arrayable_variable!(FmiBinary);
impl_typed_arrayable_variable!(FmiBinary);
impl_initializable_variable!(FmiBinary, BinaryStart);

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
