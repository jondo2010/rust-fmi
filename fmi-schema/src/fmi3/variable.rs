use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    Float32Attributes, Float64Attributes, Int16Attributes, Int32Attributes, Int8Attributes,
    IntegerBaseAttributes, RealBaseAttributes, RealVariableAttributes, UInt16Attributes,
    UInt32Attributes, UInt8Attributes,
};

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
    fn can_handle_multiple_set_per_time_instant(&self) -> bool;
    fn data_type(&self) -> VariableType;
}

pub trait ArrayableVariableTrait: AbstractVariableTrait {
    fn dimensions(&self) -> &[Dimension];
    fn intermediate_update(&self) -> bool;
    fn previous(&self) -> u32;
}

pub trait TypedArrayableVariableTrait: ArrayableVariableTrait {
    fn declared_type(&self) -> Option<&str>;
}

pub trait InitializableVariableTrait: TypedArrayableVariableTrait {
    fn initial(&self) -> Option<Initial>;
}

macro_rules! impl_abstract_variable {
    ($name:ident) => {
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
            }
            fn can_handle_multiple_set_per_time_instant(&self) -> bool {
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

macro_rules! impl_float_type {
    ($name:ident, $root:literal, $type:ty, $float_attr:ident) => {
        #[derive(Default, PartialEq, Debug, YaDeserialize)]
        #[yaserde(root = $root)]
        pub struct $name {
            #[yaserde(flatten)]
            pub base_attr: RealBaseAttributes,
            #[yaserde(flatten)]
            pub attr: $float_attr,
            #[yaserde(flatten)]
            pub init_var: InitializableVariable,
            #[yaserde(attribute, rename = "start")]
            pub start: Vec<$type>,
            #[yaserde(flatten)]
            pub real_var_attr: RealVariableAttributes,
        }

        impl_abstract_variable!($name);

        impl ArrayableVariableTrait for $name {
            fn dimensions(&self) -> &[Dimension] {
                &self.init_var.typed_arrayable_var.arrayable_var.dimensions
            }
            fn intermediate_update(&self) -> bool {
                self.init_var
                    .typed_arrayable_var
                    .arrayable_var
                    .intermediate_update
            }
            fn previous(&self) -> u32 {
                self.init_var.typed_arrayable_var.arrayable_var.previous
            }
        }

        impl TypedArrayableVariableTrait for $name {
            fn declared_type(&self) -> Option<&str> {
                self.init_var.typed_arrayable_var.declared_type.as_deref()
            }
        }

        impl InitializableVariableTrait for $name {
            fn initial(&self) -> Option<Initial> {
                self.init_var.initial
            }
        }

        impl $name {
            pub fn start(&self) -> &[$type] {
                &self.start
            }

            pub fn derivative(&self) -> Option<u32> {
                self.real_var_attr.derivative
            }

            pub fn reinit(&self) -> bool {
                self.real_var_attr.reinit
            }
        }
    };
}

macro_rules! impl_integer_type {
    ($name:ident, $root:literal, $type:ty, $int_attr:ident) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        #[yaserde(root = $root)]
        pub struct $name {
            #[yaserde(flatten)]
            pub base_attr: IntegerBaseAttributes,
            #[yaserde(flatten)]
            pub int_attr: $int_attr,
            #[yaserde(attribute)]
            pub start: $type,
            #[yaserde(flatten)]
            pub init_var: InitializableVariable,
        }

        impl_abstract_variable!($name);
    };
}

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
    #[yaserde(rename = "output")]
    Output,
    #[yaserde(rename = "local")]
    #[default]
    Local,
    /// The independent variable (usually time [but could also be, for example, angle]).
    #[yaserde(rename = "independent")]
    Independent,
    #[yaserde(rename = "dependent")]
    Dependent,
    /// The variable value can only be changed in Configuration Mode or Reconfiguration Mode.
    #[yaserde(rename = "structuredParameter")]
    StructuredParameter,
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
    Discrete,
    /// Only variables of type [`FmiFloat32`]or [`FmiFloat64`] may be continuous. The default for
    /// variables of type `FmiFloat32` and `FmiFloat64` and causality other than
    /// [`Causality::Parameter`], [`Causality::StructuredParameter`] or
    /// [`Causality::CalculatedParameter`] is continuous. Variables with variability continuous
    /// may change in Initialization Mode and in super state Initialized.
    #[yaserde(rename = "continuous")]
    #[default]
    Continuous,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Dimension {
    /// Defines a constant unsigned 64-bit integer size for this dimension. The variability of the dimension size is constant in this case.
    #[yaserde(attribute)]
    pub start: Option<u64>,
    /// If the present, it defines the size of this dimension to be the value of the variable with the value reference
    /// given by the `value_reference` attribute. The referenced variable must be a variable of type `UInt64`, and must
    /// either be a constant (i.e. with variability = constant) or a structural parameter (i.e. with causality =
    /// structuralParameter). The variability of the dimension size is in this case the variability of the referenced
    /// variable. A structural parameter must be a variable of type `UInt64` only if it is referenced in `Dimension`.
    #[yaserde(attribute, rename = "valueReference")]
    pub value_reference: Option<u32>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct AbstractVariable {
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(attribute, rename = "valueReference")]
    pub value_reference: u32,
    #[yaserde(attribute)]
    pub description: Option<String>,
    #[yaserde(attribute)]
    pub causality: Causality,
    #[yaserde(attribute)]
    pub variability: Variability,
    #[yaserde(attribute, rename = "canHandleMultipleSetPerTimeInstant")]
    pub can_handle_multiple_set_per_time_instant: bool,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct ArrayableVariable {
    #[yaserde(flatten)]
    pub abstract_var: AbstractVariable,
    /// Each `Dimension` element specifies the size of one dimension of the array
    #[yaserde(rename = "Dimension")]
    pub dimensions: Vec<Dimension>,
    #[yaserde(attribute, rename = "intermediateUpdate")]
    pub intermediate_update: bool,
    #[yaserde(attribute, rename = "previous")]
    pub previous: u32,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct TypedArrayableVariable {
    #[yaserde(flatten)]
    pub arrayable_var: ArrayableVariable,
    #[yaserde(attribute, rename = "declaredType")]
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
    #[yaserde(flatten)]
    pub typed_arrayable_var: TypedArrayableVariable,
    #[yaserde(attribute)]
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
    assert_eq!(var.can_handle_multiple_set_per_time_instant(), false);
    assert_eq!(var.intermediate_update(), false);
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
