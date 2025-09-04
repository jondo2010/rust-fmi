use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    Annotation, Annotations, Float32Attributes, Float64Attributes, Int8Attributes, Int16Attributes,
    Int32Attributes, Int64Attributes, IntegerBaseAttributes, RealBaseAttributes,
    RealVariableAttributes, UInt8Attributes, UInt16Attributes, UInt32Attributes, UInt64Attributes,
};

use crate::{Error, default_wrapper, fmi3::ModelVariables};

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

/// Append this variable to the given `ModelVariables` struct
//fn append_to_variables(self, variables: &mut ModelVariables);

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

            /// Create a new FMI variable with the given parameters
            pub fn new(
                name: String,
                value_reference: u32,
                description: Option<String>,
                causality: Causality,
                variability: Variability,
                start: Vec<$type>,
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
            #[yaserde(attribute = true)]
            pub start: Vec<$type>,
            #[yaserde(flatten = true)]
            pub init_var: InitializableVariable,
        }

        impl_abstract_variable!($name, Variability::Discrete);

        impl $name {
            /// Create a new FMI integer variable with the given parameters
            pub fn new(
                name: String,
                value_reference: u32,
                description: Option<String>,
                causality: Causality,
                variability: Variability,
                start: Vec<$type>,
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

//TODO: can this be an enum?
#[derive(Default, Clone, PartialEq, Debug, YaSerialize, YaDeserialize)]
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

impl Dimension {
    /// Create a new fixed dimension with the given size
    pub fn fixed(size: usize) -> Self {
        Self {
            start: Some(size as u64),
            value_reference: None,
        }
    }

    /// Create a new variable dimension with the given value reference
    pub fn variable(value_reference: u32) -> Self {
        Self {
            start: None,
            value_reference: Some(value_reference),
        }
    }
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
impl_arrayable_variable!(FmiInt8);
impl_typed_arrayable_variable!(FmiInt8);
impl_initializable_variable!(FmiInt8);

impl_integer_type!(FmiUInt8, "UInt8", u8, UInt8Attributes);
impl_arrayable_variable!(FmiUInt8);
impl_typed_arrayable_variable!(FmiUInt8);
impl_initializable_variable!(FmiUInt8);

impl_integer_type!(FmiInt16, "Int16", i16, Int16Attributes);
impl_arrayable_variable!(FmiInt16);
impl_typed_arrayable_variable!(FmiInt16);
impl_initializable_variable!(FmiInt16);

impl_integer_type!(FmiUInt16, "UInt16", u16, UInt16Attributes);
impl_arrayable_variable!(FmiUInt16);
impl_typed_arrayable_variable!(FmiUInt16);
impl_initializable_variable!(FmiUInt16);

impl_integer_type!(FmiInt32, "Int32", i32, Int32Attributes);
impl_arrayable_variable!(FmiInt32);
impl_typed_arrayable_variable!(FmiInt32);
impl_initializable_variable!(FmiInt32);

impl_integer_type!(FmiUInt32, "UInt32", u32, UInt32Attributes);
impl_arrayable_variable!(FmiUInt32);
impl_typed_arrayable_variable!(FmiUInt32);
impl_initializable_variable!(FmiUInt32);

impl_integer_type!(FmiInt64, "Int64", i64, Int64Attributes);
impl_arrayable_variable!(FmiInt64);
impl_typed_arrayable_variable!(FmiInt64);
impl_initializable_variable!(FmiInt64);

impl_integer_type!(FmiUInt64, "UInt64", u64, UInt64Attributes);
impl_arrayable_variable!(FmiUInt64);
impl_typed_arrayable_variable!(FmiUInt64);
impl_initializable_variable!(FmiUInt64);

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

impl FmiBoolean {
    /// Create a new FMI boolean variable with the given parameters
    pub fn new(
        name: String,
        value_reference: u32,
        description: Option<String>,
        causality: Causality,
        variability: Variability,
        start: Vec<bool>,
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

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiString {
    #[yaserde(rename = "Start")]
    pub start: Vec<StringStart>,
    #[yaserde(flatten = true)]
    pub init_var: InitializableVariable,
}

impl FmiString {
    /// Get an iterator over the start values.
    pub fn start(&self) -> impl Iterator<Item = &str> {
        self.start.iter().map(|s| s.value.as_str())
    }

    /// Create a new FMI string variable with the given parameters
    pub fn new(
        name: String,
        value_reference: u32,
        description: Option<String>,
        causality: Causality,
        variability: Variability,
        start: Vec<String>,
        initial: Option<Initial>,
    ) -> Self {
        Self {
            start: start
                .into_iter()
                .map(|value| StringStart { value })
                .collect(),
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
impl_initializable_variable!(FmiString);

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

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiBinary {
    #[yaserde(rename = "Start")]
    pub start: Vec<BinaryStart>,
    #[yaserde(attribute = true, rename = "mimeType", default = "default_mime_type")]
    pub mime_type: String,
    #[yaserde(attribute = true, rename = "maxSize")]
    pub max_size: Option<u32>,
    #[yaserde(flatten = true)]
    pub init_var: InitializableVariable,
}

fn default_mime_type() -> String {
    "application/octet-stream".into()
}

impl FmiBinary {
    /// Get an iterator over the start values.
    pub fn start(&self) -> impl Iterator<Item = &BinaryStart> {
        self.start.iter()
    }

    /// Create a new FMI binary variable with the given parameters
    pub fn new(
        name: String,
        value_reference: u32,
        description: Option<String>,
        causality: Causality,
        variability: Variability,
        start: Vec<String>,
        initial: Option<Initial>,
    ) -> Self {
        Self {
            start: start
                .into_iter()
                .map(|value| BinaryStart { value })
                .collect(),
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
impl_initializable_variable!(FmiBinary);

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
        assert_eq!(var.start, vec![0]);
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

    #[test]
    fn test_binary() {
        let xml = r#"
            <Binary name="Binary_input" valueReference="31" causality="input">
                <Start value="666f6f"/>
            </Binary>"#;

        let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "Binary_input");
        assert_eq!(var.value_reference(), 31);
        assert_eq!(var.causality(), Causality::Input);
        let start0 = var.start().next().unwrap();
        assert_eq!(start0.value.as_str(), "666f6f");
        assert_eq!(start0.as_bytes(), Ok(vec![0x66, 0x6f, 0x6f]));
    }

    #[test]
    fn test_float32() {
        let xml =
            r#"<Float32 name="float32_var" valueReference="10" causality="output" start="3.14"/>"#;
        let var: FmiFloat32 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "float32_var");
        assert_eq!(var.value_reference(), 10);
        assert_eq!(var.causality(), Causality::Output);
        assert_eq!(var.start(), &[3.14]);
        assert_eq!(var.variability(), Variability::Continuous); // Default for float types
        assert_eq!(var.derivative(), None);
        assert_eq!(var.reinit(), None);
    }

    #[test]
    fn test_int8() {
        let xml = r#"<Int8 name="int8_var" valueReference="20" causality="parameter" variability="fixed" start="-128"/>"#;
        let var: FmiInt8 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "int8_var");
        assert_eq!(var.value_reference(), 20);
        assert_eq!(var.causality(), Causality::Parameter);
        assert_eq!(var.start, vec![-128]);
        assert_eq!(var.variability(), Variability::Fixed);
    }

    #[test]
    fn test_uint8() {
        let xml = r#"<UInt8 name="uint8_var" valueReference="21" causality="local" start="255"/>"#;
        let var: FmiUInt8 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "uint8_var");
        assert_eq!(var.value_reference(), 21);
        assert_eq!(var.causality(), Causality::Local);
        assert_eq!(var.start, vec![255]);
        assert_eq!(var.variability(), Variability::Discrete); // Default for integer types
    }

    #[test]
    fn test_uint16() {
        let xml = r#"<UInt16 name="uint16_var" valueReference="22" causality="calculatedParameter" start="65535"/>"#;
        let var: FmiUInt16 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "uint16_var");
        assert_eq!(var.value_reference(), 22);
        assert_eq!(var.causality(), Causality::CalculatedParameter);
        assert_eq!(var.start, vec![65535]);
    }

    #[test]
    fn test_int32() {
        let xml = r#"<Int32 name="int32_var" valueReference="23" causality="structuralParameter" variability="tunable" start="-2147483648"/>"#;
        let var: FmiInt32 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "int32_var");
        assert_eq!(var.value_reference(), 23);
        assert_eq!(var.causality(), Causality::StructuralParameter);
        assert_eq!(var.start, vec![-2147483648]);
        assert_eq!(var.variability(), Variability::Tunable);
    }

    #[test]
    fn test_uint32() {
        let xml = r#"<UInt32 name="uint32_var" valueReference="24" causality="independent" start="4294967295"/>"#;
        let var: FmiUInt32 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "uint32_var");
        assert_eq!(var.value_reference(), 24);
        assert_eq!(var.causality(), Causality::Independent);
        assert_eq!(var.start, vec![4294967295]);
    }

    #[test]
    fn test_int64() {
        let xml = r#"<Int64 name="int64_var" valueReference="25" causality="dependent" start="-9223372036854775808"/>"#;
        let var: FmiInt64 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "int64_var");
        assert_eq!(var.value_reference(), 25);
        assert_eq!(var.causality(), Causality::Dependent);
        assert_eq!(var.start, vec![-9223372036854775808]);
    }

    #[test]
    fn test_uint64() {
        let xml = r#"<UInt64 name="uint64_var" valueReference="26" causality="input" variability="constant" start="18446744073709551615"/>"#;
        let var: FmiUInt64 = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "uint64_var");
        assert_eq!(var.value_reference(), 26);
        assert_eq!(var.causality(), Causality::Input);
        assert_eq!(var.start, vec![18446744073709551615]);
        assert_eq!(var.variability(), Variability::Constant);
    }

    #[test]
    fn test_boolean() {
        let xml = r#"<Boolean name="boolean_var" valueReference="30" causality="output" start="true false true"/>"#;
        let var: FmiBoolean = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.name(), "boolean_var");
        assert_eq!(var.value_reference(), 30);
        assert_eq!(var.causality(), Causality::Output);
        assert_eq!(var.start, vec![true, false, true]);
        assert_eq!(var.variability(), Variability::Discrete); // Default for boolean
    }

    #[test]
    fn test_variable_with_all_attributes() {
        let xml = r#"<Float64
            name="complex_var"
            valueReference="100"
            description="A complex variable with many attributes"
            causality="output"
            variability="continuous"
            canHandleMultipleSetPerTimeInstant="true"
            intermediateUpdate="false"
            previous="99"
            initial="calculated"
            declaredType="CustomType"
            start="1.0 2.0"
            derivative="101"
            reinit="true">
            <Dimension start="2"/>
        </Float64>"#;

        let var: FmiFloat64 = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "complex_var");
        assert_eq!(var.value_reference(), 100);
        assert_eq!(
            var.description(),
            Some("A complex variable with many attributes")
        );
        assert_eq!(var.causality(), Causality::Output);
        assert_eq!(var.variability(), Variability::Continuous);
        assert_eq!(var.can_handle_multiple_set_per_time_instant(), Some(true));
        assert_eq!(var.intermediate_update(), Some(false));
        assert_eq!(var.previous(), Some(99));
        assert_eq!(var.initial(), Some(Initial::Calculated));
        assert_eq!(var.declared_type(), Some("CustomType"));
        assert_eq!(var.start(), &[1.0, 2.0]);
        assert_eq!(var.derivative(), Some(101));
        assert_eq!(var.reinit(), Some(true));
        assert_eq!(var.dimensions().len(), 1);
        assert_eq!(var.dimensions()[0].start, Some(2));
    }

    #[test]
    fn test_dimension_with_value_reference() {
        let xml = r#"<Float32
            name="matrix_var"
            valueReference="200"
            causality="parameter"
            start="1.0 2.0 3.0 4.0">
            <Dimension valueReference="201"/>
            <Dimension start="2"/>
        </Float32>"#;

        let var: FmiFloat32 = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "matrix_var");
        assert_eq!(var.dimensions().len(), 2);
        assert_eq!(var.dimensions()[0].value_reference, Some(201));
        assert_eq!(var.dimensions()[0].start, None);
        assert_eq!(var.dimensions()[1].value_reference, None);
        assert_eq!(var.dimensions()[1].start, Some(2));
        assert_eq!(var.start(), &[1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_string_multiple_starts() {
        let xml = r#"<String name="multi_string" valueReference="300" causality="parameter">
            <Start value="First string"/>
            <Start value="Second string"/>
            <Start value="Third string"/>
        </String>"#;

        let var: FmiString = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "multi_string");
        let start_values: Vec<&str> = var.start().collect();
        assert_eq!(
            start_values,
            vec!["First string", "Second string", "Third string"]
        );
    }

    #[test]
    fn test_binary_multiple_starts_and_attributes() {
        let xml = r#"<Binary
            name="multi_binary"
            valueReference="400"
            causality="input"
            mimeType="application/custom"
            maxSize="1024">
            <Start value="48656c6c6f"/>
            <Start value="576f726c64"/>
        </Binary>"#;

        let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "multi_binary");
        assert_eq!(var.mime_type, "application/custom");
        assert_eq!(var.max_size, Some(1024));

        let start_values: Vec<&BinaryStart> = var.start().collect();
        assert_eq!(start_values.len(), 2);
        assert_eq!(start_values[0].value, "48656c6c6f");
        assert_eq!(start_values[1].value, "576f726c64");

        // Test hex parsing
        assert_eq!(
            start_values[0].as_bytes(),
            Ok(vec![0x48, 0x65, 0x6c, 0x6c, 0x6f])
        ); // "Hello"
        assert_eq!(
            start_values[1].as_bytes(),
            Ok(vec![0x57, 0x6f, 0x72, 0x6c, 0x64])
        ); // "World"
    }

    #[test]
    fn test_binary_hex_parsing_with_prefix() {
        let xml = r#"<Binary name="hex_binary" valueReference="500" causality="input">
            <Start value="0x48656C6C6F"/>
        </Binary>"#;

        let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
        let start0 = var.start().next().unwrap();
        assert_eq!(start0.as_bytes(), Ok(vec![0x48, 0x65, 0x6C, 0x6C, 0x6F])); // "HeLLO"
    }

    #[test]
    fn test_binary_hex_parsing_with_whitespace() {
        let xml = r#"<Binary name="spaced_binary" valueReference="600" causality="input">
            <Start value="48 65 6c 6c 6f 20 57 6f 72 6c 64"/>
        </Binary>"#;

        let var: FmiBinary = yaserde::de::from_str(xml).unwrap();
        let start0 = var.start().next().unwrap();
        assert_eq!(
            start0.as_bytes(),
            Ok(vec![
                0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64
            ])
        ); // "Hello World"
    }

    #[test]
    fn test_initial_values() {
        let xml_exact =
            r#"<Float64 name="exact_var" valueReference="700" initial="exact" start="1.0"/>"#;
        let var_exact: FmiFloat64 = yaserde::de::from_str(xml_exact).unwrap();
        assert_eq!(var_exact.initial(), Some(Initial::Exact));

        let xml_approx =
            r#"<Float64 name="approx_var" valueReference="701" initial="approx" start="1.0"/>"#;
        let var_approx: FmiFloat64 = yaserde::de::from_str(xml_approx).unwrap();
        assert_eq!(var_approx.initial(), Some(Initial::Approx));

        let xml_calculated =
            r#"<Float64 name="calc_var" valueReference="702" initial="calculated" start="1.0"/>"#;
        let var_calculated: FmiFloat64 = yaserde::de::from_str(xml_calculated).unwrap();
        assert_eq!(var_calculated.initial(), Some(Initial::Calculated));
    }

    #[test]
    fn test_variable_annotations() {
        let xml = r#"<Int32 name="annotated_var" valueReference="800" causality="local" start="42">
            <Annotations>
                <Annotation type="info">This is an informational annotation.</Annotation>
                <Annotation type="warning">This is a warning annotation.</Annotation>
            </Annotations>
        </Int32>"#;

        let var: FmiInt32 = yaserde::de::from_str(xml).unwrap();
        assert_eq!(var.name(), "annotated_var");
        assert_eq!(var.value_reference(), 800);
        assert_eq!(var.causality(), Causality::Local);
        assert_eq!(var.start, vec![42]);

        let annotations = var.annotations().unwrap();
        assert_eq!(annotations.annotations.len(), 2);
        assert_eq!(annotations.annotations[0].r#type, "info".to_string());
        assert_eq!(
            annotations.annotations[0].content,
            "This is an informational annotation."
        );
        assert_eq!(annotations.annotations[1].r#type, "warning".to_string());
        assert_eq!(
            annotations.annotations[1].content,
            "This is a warning annotation."
        );
    }

    #[test]
    fn test_data_type_enum() {
        let float32_var: FmiFloat32 = Default::default();
        assert_eq!(float32_var.data_type(), VariableType::FmiFloat32);

        let float64_var: FmiFloat64 = Default::default();
        assert_eq!(float64_var.data_type(), VariableType::FmiFloat64);

        let int8_var: FmiInt8 = Default::default();
        assert_eq!(int8_var.data_type(), VariableType::FmiInt8);

        let uint8_var: FmiUInt8 = Default::default();
        assert_eq!(uint8_var.data_type(), VariableType::FmiUInt8);

        let int16_var: FmiInt16 = Default::default();
        assert_eq!(int16_var.data_type(), VariableType::FmiInt16);

        let uint16_var: FmiUInt16 = Default::default();
        assert_eq!(uint16_var.data_type(), VariableType::FmiUInt16);

        let int32_var: FmiInt32 = Default::default();
        assert_eq!(int32_var.data_type(), VariableType::FmiInt32);

        let uint32_var: FmiUInt32 = Default::default();
        assert_eq!(uint32_var.data_type(), VariableType::FmiUInt32);

        let int64_var: FmiInt64 = Default::default();
        assert_eq!(int64_var.data_type(), VariableType::FmiInt64);

        let uint64_var: FmiUInt64 = Default::default();
        assert_eq!(uint64_var.data_type(), VariableType::FmiUInt64);

        let boolean_var: FmiBoolean = Default::default();
        assert_eq!(boolean_var.data_type(), VariableType::FmiBoolean);

        let string_var: FmiString = Default::default();
        assert_eq!(string_var.data_type(), VariableType::FmiString);

        let binary_var: FmiBinary = Default::default();
        assert_eq!(binary_var.data_type(), VariableType::FmiBinary);
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn test_arrow_data_type_conversion() {
        use arrow::datatypes::DataType;

        assert_eq!(DataType::from(VariableType::FmiFloat32), DataType::Float32);
        assert_eq!(DataType::from(VariableType::FmiFloat64), DataType::Float64);
        assert_eq!(DataType::from(VariableType::FmiInt8), DataType::Int8);
        assert_eq!(DataType::from(VariableType::FmiUInt8), DataType::UInt8);
        assert_eq!(DataType::from(VariableType::FmiInt16), DataType::Int16);
        assert_eq!(DataType::from(VariableType::FmiUInt16), DataType::UInt16);
        assert_eq!(DataType::from(VariableType::FmiInt32), DataType::Int32);
        assert_eq!(DataType::from(VariableType::FmiUInt32), DataType::UInt32);
        assert_eq!(DataType::from(VariableType::FmiInt64), DataType::Int64);
        assert_eq!(DataType::from(VariableType::FmiUInt64), DataType::UInt64);
        assert_eq!(DataType::from(VariableType::FmiBoolean), DataType::Boolean);
        assert_eq!(DataType::from(VariableType::FmiString), DataType::Utf8);
        assert_eq!(DataType::from(VariableType::FmiBinary), DataType::Binary);
    }
}
