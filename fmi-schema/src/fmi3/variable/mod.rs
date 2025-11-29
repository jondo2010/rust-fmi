use super::Annotations;

use crate::utils::AttrList;

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

mod dimension;
mod model_variables;
#[cfg(test)]
mod tests;

pub use dimension::Dimension;
pub use model_variables::{AppendToModelVariables, ModelVariables, Variable};

/// Base type for variable aliases
#[derive(PartialEq, Debug, Default, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Alias")]
pub struct VariableAlias {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
}

/// Alias for float variables (Float32 and Float64) with additional displayUnit attribute
#[derive(PartialEq, Debug, Default, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Alias")]
pub struct FloatVariableAlias {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(attr = "displayUnit")]
    pub display_unit: Option<String>,
}

/// IntervalVariability declares the Clock type
///
/// See <https://fmi-standard.org/docs/3.0.1/#table-overview-clocks>
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum IntervalVariability {
    Constant,
    Fixed,
    Tunable,
    Changing,
    Countdown,
    /// IntervalVariability *must* be set to Triggered for Clocks with causality = Output
    #[default]
    Triggered,
}

impl FromStr for IntervalVariability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constant" => Ok(Self::Constant),
            "fixed" => Ok(Self::Fixed),
            "tunable" => Ok(Self::Tunable),
            "changing" => Ok(Self::Changing),
            "countdown" => Ok(Self::Countdown),
            "triggered" => Ok(Self::Triggered),
            _ => Err(format!("Invalid IntervalVariability: {}", s)),
        }
    }
}

impl Display for IntervalVariability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Constant => "constant",
            Self::Fixed => "fixed",
            Self::Tunable => "tunable",
            Self::Changing => "changing",
            Self::Countdown => "countdown",
            Self::Triggered => "triggered",
        };
        write!(f, "{}", s)
    }
}

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
    FmiClock,
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
            VariableType::FmiClock => arrow::datatypes::DataType::Boolean,
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
    fn clocks(&self) -> Option<&[u32]>;
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

pub trait TypedVariableTrait {
    fn declared_type(&self) -> Option<&str>;
}

pub trait TypedArrayableVariableTrait: ArrayableVariableTrait + TypedVariableTrait {}
impl<T> TypedArrayableVariableTrait for T where T: ArrayableVariableTrait + TypedVariableTrait {}

pub trait InitializableVariableTrait {
    type StartType;
    fn initial(&self) -> Option<Initial>;

    fn start(&self) -> Option<&[Self::StartType]>;
}

macro_rules! impl_abstract_variable {
    ($name: ident, $default_variability: expr, $variable_type: expr) => {
        impl AbstractVariableTrait for $name {
            fn name(&self) -> &str {
                &self.name
            }
            fn value_reference(&self) -> u32 {
                self.value_reference
            }
            fn description(&self) -> Option<&str> {
                self.description.as_deref()
            }
            fn causality(&self) -> Causality {
                self.causality.unwrap_or_default()
            }
            fn variability(&self) -> Variability {
                self.variability.unwrap_or($default_variability)
            }
            fn can_handle_multiple_set_per_time_instant(&self) -> Option<bool> {
                self.can_handle_multiple_set_per_time_instant
            }
            fn clocks(&self) -> Option<&[u32]> {
                self.clocks.as_ref().map(|c| c.0.as_slice())
            }
            fn data_type(&self) -> VariableType {
                $variable_type
            }
            fn annotations(&self) -> Option<&Annotations> {
                self.annotations.as_ref()
            }
        }
    };
}

macro_rules! impl_arrayable_variable {
    ($name:ident) => {
        impl ArrayableVariableTrait for $name {
            fn dimensions(&self) -> &[Dimension] {
                &self.dimensions
            }
            fn add_dimensions(&mut self, dims: &[Dimension]) {
                self.dimensions.extend_from_slice(dims);
            }
            fn intermediate_update(&self) -> Option<bool> {
                self.intermediate_update
            }
            fn previous(&self) -> Option<u32> {
                self.previous
            }
        }
    };
}

macro_rules! impl_typed_variable {
    ($name:ident) => {
        impl TypedVariableTrait for $name {
            fn declared_type(&self) -> Option<&str> {
                self.declared_type.as_deref()
            }
        }
    };
}

macro_rules! impl_initializable_variable {
    ($name: ident, $type: ty) => {
        impl InitializableVariableTrait for $name {
            type StartType = $type;

            fn initial(&self) -> Option<Initial> {
                self.initial
            }

            fn start(&self) -> Option<&[Self::StartType]> {
                self.start.as_ref().map(|s| s.0.as_slice())
            }
        }
    };
}

macro_rules! impl_float_type {
    // Implementation for float types using hard_xml derives
    ($name:ident, $tag:expr, $type:ty, $variable_type:expr) => {
        #[derive(PartialEq, Debug, Default, hard_xml::XmlRead, hard_xml::XmlWrite)]
        #[xml(tag = $tag, strict(unknown_attribute, unknown_element))]
        pub struct $name {
            #[xml(attr = "name")]
            pub name: String,
            #[xml(attr = "valueReference")]
            pub value_reference: u32,
            #[xml(attr = "description")]
            pub description: Option<String>,
            #[xml(attr = "causality")]
            pub causality: Option<Causality>,
            #[xml(attr = "variability")]
            pub variability: Option<Variability>,
            #[xml(attr = "canHandleMultipleSetPerTimeInstant")]
            pub can_handle_multiple_set_per_time_instant: Option<bool>,
            #[xml(attr = "clocks")]
            pub clocks: Option<AttrList<u32>>,
            #[xml(attr = "declaredType")]
            pub declared_type: Option<String>,
            #[xml(child = "Dimension")]
            pub dimensions: Vec<Dimension>,
            #[xml(attr = "intermediateUpdate")]
            pub intermediate_update: Option<bool>,
            #[xml(attr = "previous")]
            pub previous: Option<u32>,
            /// Initial or guess value of the variable. During instantiation, the FMU initializes its variables with their start values.
            #[xml(attr = "start")]
            pub start: Option<AttrList<$type>>,
            #[xml(attr = "initial")]
            pub initial: Option<Initial>,
            #[xml(attr = "min")]
            pub min: Option<$type>,
            #[xml(attr = "max")]
            pub max: Option<$type>,
            #[xml(attr = "derivative")]
            pub derivative: Option<u32>,
            #[xml(attr = "reinit")]
            pub reinit: Option<bool>,
            #[xml(child = "Annotations")]
            pub annotations: Option<Annotations>,
            #[xml(child = "Alias")]
            pub aliases: Vec<FloatVariableAlias>,
        }

        impl_abstract_variable!($name, Variability::Continuous, $variable_type);
        impl_arrayable_variable!($name);
        impl_typed_variable!($name);
        impl_initializable_variable!($name, $type);

        impl $name {
            pub fn derivative(&self) -> Option<u32> {
                self.derivative
            }

            pub fn reinit(&self) -> Option<bool> {
                self.reinit
            }

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
                    name,
                    value_reference,
                    description,
                    causality: Some(causality),
                    variability: Some(variability),
                    start: start.map(AttrList),
                    initial,
                    ..Default::default()
                }
            }
        }
    };
}

macro_rules! impl_integer_type {
    // Implementation for integer types using hard_xml derives
    ($name:ident, $tag:expr, $type:ty, $variable_type:expr) => {
        #[derive(PartialEq, Debug, Default, hard_xml::XmlRead, hard_xml::XmlWrite)]
        #[xml(tag = $tag, strict(unknown_attribute, unknown_element))]
        pub struct $name {
            #[xml(attr = "name")]
            pub name: String,
            #[xml(attr = "valueReference")]
            pub value_reference: u32,
            #[xml(attr = "description")]
            pub description: Option<String>,
            #[xml(attr = "causality")]
            pub causality: Option<Causality>,
            #[xml(attr = "variability")]
            pub variability: Option<Variability>,
            #[xml(attr = "canHandleMultipleSetPerTimeInstant")]
            pub can_handle_multiple_set_per_time_instant: Option<bool>,
            #[xml(attr = "clocks")]
            pub clocks: Option<AttrList<u32>>,
            #[xml(attr = "declaredType")]
            pub declared_type: Option<String>,
            #[xml(child = "Dimension")]
            pub dimensions: Vec<Dimension>,
            #[xml(attr = "intermediateUpdate")]
            pub intermediate_update: Option<bool>,
            #[xml(attr = "previous")]
            pub previous: Option<u32>,
            /// Initial or guess value of the variable. During instantiation, the FMU initializes its variables with their start values.
            #[xml(attr = "start")]
            pub start: Option<AttrList<$type>>,
            #[xml(attr = "initial")]
            pub initial: Option<Initial>,
            #[xml(attr = "min")]
            pub min: Option<$type>,
            #[xml(attr = "max")]
            pub max: Option<$type>,
            #[xml(attr = "quantity")]
            pub quantity: Option<String>,
            #[xml(child = "Annotations")]
            pub annotations: Option<Annotations>,
            #[xml(child = "Alias")]
            pub aliases: Vec<VariableAlias>,
        }

        impl_abstract_variable!($name, Variability::Discrete, $variable_type);
        impl_arrayable_variable!($name);
        impl_typed_variable!($name);
        impl_initializable_variable!($name, $type);

        impl $name {
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
                    name,
                    value_reference,
                    description,
                    causality: Some(causality),
                    variability: Some(variability),
                    start: start.map(AttrList),
                    initial,
                    ..Default::default()
                }
            }
        }
    };
}

/// Enumeration that defines the causality of the variable.
///
/// See <https://fmi-standard.org/docs/3.0.1/#causality>
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Causality {
    /// A data value that is constant during the simulation
    Parameter,
    /// A data value that is constant during the simulation and is computed during initialization
    /// or when tunable parameters change.
    CalculatedParameter,
    /// The variable value can be provided by the importer.
    Input,
    /// The values of these variables are computed in the FMU and they are designed to be used outside the FMU.
    Output,
    /// Local variables of the FMU that must not be used for FMU connections
    #[default]
    Local,
    /// The independent variable (usually time [but could also be, for example, angle]).
    Independent,
    Dependent,
    /// The variable value can only be changed in Configuration Mode or Reconfiguration Mode.
    StructuralParameter,
}

impl FromStr for Causality {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parameter" => Ok(Causality::Parameter),
            "calculatedParameter" => Ok(Causality::CalculatedParameter),
            "input" => Ok(Causality::Input),
            "output" => Ok(Causality::Output),
            "local" => Ok(Causality::Local),
            "independent" => Ok(Causality::Independent),
            "dependent" => Ok(Causality::Dependent),
            "structuralParameter" => Ok(Causality::StructuralParameter),
            _ => Err(format!("Invalid Causality: {}", s)),
        }
    }
}

impl Display for Causality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Causality::Parameter => write!(f, "parameter"),
            Causality::CalculatedParameter => write!(f, "calculatedParameter"),
            Causality::Input => write!(f, "input"),
            Causality::Output => write!(f, "output"),
            Causality::Local => write!(f, "local"),
            Causality::Independent => write!(f, "independent"),
            Causality::Dependent => write!(f, "dependent"),
            Causality::StructuralParameter => write!(f, "structuralParameter"),
        }
    }
}

/// Enumeration that defines the time dependency of the variable, in other words, it defines the
/// time instants when a variable may be changed by the importer or may change its value due to FMU
/// internal computations, depending on their causality.
///
/// See <https://fmi-standard.org/docs/3.0.1/#variability>
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Variability {
    /// The value of the variable never changes.
    Constant,
    /// The value of the variable is fixed in super state Initialized, in other words, after
    /// [`exit_initialization_mode()`] was called the variable value does not change anymore. The
    /// default for variables of causality [`Causality::Parameter`],
    /// [`Causality::StructuredParameter`] or [`Causality::CalculatedParameter`] is `Fixed`.
    Fixed,
    /// The value of the variable is constant between events (ME and CS if Event Mode is supported)
    /// and between communication points (CS and SE). A parameter with variability = tunable
    /// may be changed only in Event Mode or, if Event Mode is not supported, at communication
    /// points (CS and SE).
    Tunable,
    /// * Model Exchange: The value of the variable may change only in Event Mode.
    /// * Co-Simulation: If Event Mode is used (see `event_mode_used`), the value of the variable
    ///   may only change in Event Mode. If Event Mode is not used, the value may change at
    ///   communication points and the FMU must detect and handle such events internally. During
    ///   Intermediate Update Mode, discrete variables are not allowed to change.
    /// * Scheduled Execution: The value may change only at communication points.
    #[default]
    Discrete,
    /// Only variables of type [`FmiFloat32`]or [`FmiFloat64`] may be continuous. The default for
    /// variables of type `FmiFloat32` and `FmiFloat64` and causality other than
    /// [`Causality::Parameter`], [`Causality::StructuredParameter`] or
    /// [`Causality::CalculatedParameter`] is continuous. Variables with variability continuous
    /// may change in Initialization Mode and in super state Initialized.
    Continuous,
}

impl FromStr for Variability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "constant" => Ok(Variability::Constant),
            "fixed" => Ok(Variability::Fixed),
            "tunable" => Ok(Variability::Tunable),
            "discrete" => Ok(Variability::Discrete),
            "continuous" => Ok(Variability::Continuous),
            _ => Err(format!("Invalid Variability: {}", s)),
        }
    }
}

impl Display for Variability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variability::Constant => write!(f, "constant"),
            Variability::Fixed => write!(f, "fixed"),
            Variability::Tunable => write!(f, "tunable"),
            Variability::Discrete => write!(f, "discrete"),
            Variability::Continuous => write!(f, "continuous"),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Initial {
    #[default]
    Exact,
    Approx,
    Calculated,
}

impl FromStr for Initial {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "exact" => Ok(Initial::Exact),
            "approx" => Ok(Initial::Approx),
            "calculated" => Ok(Initial::Calculated),
            _ => Err(format!("Invalid Initial: {}", s)),
        }
    }
}

impl Display for Initial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Initial::Exact => write!(f, "exact"),
            Initial::Approx => write!(f, "approx"),
            Initial::Calculated => write!(f, "calculated"),
        }
    }
}

impl_float_type!(FmiFloat32, "Float32", f32, VariableType::FmiFloat32);
impl_float_type!(FmiFloat64, "Float64", f64, VariableType::FmiFloat64);
impl_integer_type!(FmiInt8, "Int8", i8, VariableType::FmiInt8);
impl_integer_type!(FmiUInt8, "UInt8", u8, VariableType::FmiUInt8);
impl_integer_type!(FmiInt16, "Int16", i16, VariableType::FmiInt16);
impl_integer_type!(FmiUInt16, "UInt16", u16, VariableType::FmiUInt16);
impl_integer_type!(FmiInt32, "Int32", i32, VariableType::FmiInt32);
impl_integer_type!(FmiUInt32, "UInt32", u32, VariableType::FmiUInt32);
impl_integer_type!(FmiInt64, "Int64", i64, VariableType::FmiInt64);
impl_integer_type!(FmiUInt64, "UInt64", u64, VariableType::FmiUInt64);

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Boolean", strict(unknown_attribute, unknown_element))]
pub struct FmiBoolean {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "valueReference")]
    pub value_reference: u32,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(attr = "causality")]
    pub causality: Option<Causality>,
    #[xml(attr = "variability")]
    pub variability: Option<Variability>,
    #[xml(attr = "canHandleMultipleSetPerTimeInstant")]
    pub can_handle_multiple_set_per_time_instant: Option<bool>,
    #[xml(attr = "clocks")]
    pub clocks: Option<AttrList<u32>>,
    #[xml(attr = "declaredType")]
    pub declared_type: Option<String>,
    #[xml(child = "Dimension")]
    pub dimensions: Vec<Dimension>,
    #[xml(attr = "intermediateUpdate")]
    pub intermediate_update: Option<bool>,
    #[xml(attr = "previous")]
    pub previous: Option<u32>,
    #[xml(attr = "initial")]
    pub initial: Option<Initial>,
    #[xml(attr = "start")]
    pub start: Option<AttrList<bool>>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(child = "Alias")]
    pub aliases: Vec<VariableAlias>,
}

impl_abstract_variable!(FmiBoolean, Variability::Discrete, VariableType::FmiBoolean);
impl_arrayable_variable!(FmiBoolean);
impl_typed_variable!(FmiBoolean);
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
            start: start.map(AttrList),
            name,
            value_reference,
            description,
            causality: Some(causality),
            variability: Some(variability),
            can_handle_multiple_set_per_time_instant: None,
            clocks: None,
            annotations: None,
            dimensions: vec![],
            intermediate_update: None,
            previous: None,
            declared_type: None,
            initial,
            aliases: vec![],
        }
    }
}

/// A String start value element
#[derive(PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Start")]
pub struct StringStart {
    #[xml(attr = "value")]
    pub value: String,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "String", strict(unknown_attribute, unknown_element))]
pub struct FmiString {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "valueReference")]
    pub value_reference: u32,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(attr = "causality")]
    pub causality: Option<Causality>,
    #[xml(attr = "variability")]
    pub variability: Option<Variability>,
    #[xml(attr = "canHandleMultipleSetPerTimeInstant")]
    pub can_handle_multiple_set_per_time_instant: Option<bool>,
    #[xml(attr = "clocks")]
    pub clocks: Option<AttrList<u32>>,
    #[xml(attr = "declaredType")]
    pub declared_type: Option<String>,
    #[xml(child = "Dimension")]
    pub dimensions: Vec<Dimension>,
    #[xml(attr = "intermediateUpdate")]
    pub intermediate_update: Option<bool>,
    #[xml(attr = "previous")]
    pub previous: Option<u32>,
    /// Initial or guess value of the variable. During instantiation, the FMU initializes its variables with their start values.
    #[xml(child = "Start")]
    pub start: Vec<StringStart>,
    #[xml(attr = "initial")]
    pub initial: Option<Initial>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(child = "Alias")]
    pub aliases: Vec<VariableAlias>,
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
            start: start
                .unwrap_or_default()
                .into_iter()
                .map(|value| StringStart { value })
                .collect(),
            name,
            value_reference,
            description,
            causality: Some(causality),
            variability: Some(variability),
            can_handle_multiple_set_per_time_instant: None,
            clocks: None,
            annotations: None,
            dimensions: vec![],
            intermediate_update: None,
            previous: None,
            declared_type: None,
            initial,
            aliases: vec![],
        }
    }
}

impl_abstract_variable!(FmiString, Variability::Discrete, VariableType::FmiString);
impl_arrayable_variable!(FmiString);
impl_typed_variable!(FmiString);
impl InitializableVariableTrait for FmiString {
    type StartType = StringStart;

    fn initial(&self) -> Option<Initial> {
        self.initial
    }

    fn start(&self) -> Option<&[Self::StartType]> {
        if self.start.is_empty() {
            None
        } else {
            Some(&self.start)
        }
    }
}

#[derive(PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Start")]
pub struct BinaryStart {
    #[xml(attr = "value")]
    pub value: String,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Binary")]
pub struct FmiBinary {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "valueReference")]
    pub value_reference: u32,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(attr = "causality")]
    pub causality: Option<Causality>,
    #[xml(attr = "variability")]
    pub variability: Option<Variability>,
    #[xml(attr = "canHandleMultipleSetPerTimeInstant")]
    pub can_handle_multiple_set_per_time_instant: Option<bool>,
    #[xml(attr = "clocks")]
    pub clocks: Option<AttrList<u32>>,
    #[xml(attr = "declaredType")]
    pub declared_type: Option<String>,
    #[xml(child = "Dimension")]
    pub dimensions: Vec<Dimension>,
    #[xml(attr = "intermediateUpdate")]
    pub intermediate_update: Option<bool>,
    #[xml(attr = "previous")]
    pub previous: Option<u32>,
    #[xml(child = "Start")]
    pub start: Vec<BinaryStart>,
    #[xml(attr = "mimeType")]
    pub mime_type: Option<String>,
    #[xml(attr = "maxSize")]
    pub max_size: Option<u32>,
    #[xml(attr = "initial")]
    pub initial: Option<Initial>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(child = "Alias")]
    pub aliases: Vec<VariableAlias>,
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
            start: start
                .unwrap_or_default()
                .into_iter()
                .map(|value| BinaryStart { value })
                .collect(),
            mime_type: Some(default_mime_type()),
            max_size: None,
            name,
            value_reference,
            description,
            causality: Some(causality),
            variability: Some(variability),
            can_handle_multiple_set_per_time_instant: None,
            clocks: None,
            annotations: None,
            dimensions: vec![],
            intermediate_update: None,
            previous: None,
            declared_type: None,
            initial,
            aliases: vec![],
        }
    }

    /// Decode a single hex-encoded start value string into bytes.
    /// Handles optional "0x" prefix and whitespace in the hex string.
    pub fn decode_start_value(hex_string: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
        // Remove any whitespace and 0x prefix
        let cleaned = hex_string.replace(|c: char| c.is_whitespace(), "");
        let hex_str = cleaned.strip_prefix("0x").unwrap_or(&cleaned);

        // Parse pairs of hex digits
        (0..hex_str.len())
            .step_by(2)
            .map(|i| {
                let byte_str = &hex_str[i..std::cmp::min(i + 2, hex_str.len())];
                u8::from_str_radix(byte_str, 16)
            })
            .collect()
    }
}

impl_abstract_variable!(FmiBinary, Variability::Discrete, VariableType::FmiBinary);
impl_arrayable_variable!(FmiBinary);
impl_typed_variable!(FmiBinary);

impl InitializableVariableTrait for FmiBinary {
    type StartType = BinaryStart;

    fn initial(&self) -> Option<Initial> {
        self.initial
    }

    fn start(&self) -> Option<&[Self::StartType]> {
        if self.start.is_empty() {
            None
        } else {
            Some(&self.start)
        }
    }
}

/// Clock variable type
#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Clock", strict(unknown_attribute, unknown_element))]
pub struct FmiClock {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "valueReference")]
    pub value_reference: u32,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(attr = "causality")]
    pub causality: Option<Causality>,
    #[xml(attr = "variability")]
    pub variability: Option<Variability>,
    #[xml(attr = "canHandleMultipleSetPerTimeInstant")]
    pub can_handle_multiple_set_per_time_instant: Option<bool>,
    #[xml(attr = "clocks")]
    pub clocks: Option<AttrList<u32>>,
    #[xml(attr = "declaredType")]
    pub declared_type: Option<String>,
    #[xml(attr = "canBeDeactivated")]
    pub can_be_deactivated: Option<bool>,
    #[xml(attr = "priority")]
    pub priority: Option<i32>,
    #[xml(attr = "intervalVariability")]
    pub interval_variability: Option<IntervalVariability>,
    #[xml(attr = "intervalDecimal")]
    pub interval_decimal: Option<f64>,
    #[xml(attr = "shiftDecimal")]
    pub shift_decimal: Option<f64>,
    #[xml(attr = "supportsFraction")]
    pub supports_fraction: Option<bool>,
    #[xml(attr = "resolution")]
    pub resolution: Option<u64>,
    #[xml(attr = "intervalCounter")]
    pub interval_counter: Option<u64>,
    #[xml(attr = "shiftCounter")]
    pub shift_counter: Option<u64>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(child = "Alias")]
    pub aliases: Vec<VariableAlias>,
}

impl_abstract_variable!(FmiClock, Variability::Discrete, VariableType::FmiClock);
impl_typed_variable!(FmiClock);

impl FmiClock {
    pub fn new(
        name: String,
        value_reference: u32,
        description: Option<String>,
        causality: Causality,
        variability: Variability,
    ) -> Self {
        Self {
            name,
            value_reference,
            description,
            causality: Some(causality),
            variability: Some(variability),
            can_handle_multiple_set_per_time_instant: None,
            clocks: None,
            declared_type: None,
            can_be_deactivated: None,
            priority: None,
            interval_variability: None,
            interval_decimal: None,
            shift_decimal: None,
            supports_fraction: None,
            resolution: None,
            interval_counter: None,
            shift_counter: None,
            annotations: None,
            aliases: vec![],
        }
    }

    pub fn interval_variability(&self) -> IntervalVariability {
        self.interval_variability.unwrap_or_default()
    }
}
