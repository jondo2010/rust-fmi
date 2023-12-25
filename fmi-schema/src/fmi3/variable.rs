use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    Float32Attributes, Float64Attributes, Int16Attributes, Int32Attributes, Int8Attributes,
    IntegerBaseAttributes, RealBaseAttributes, RealVariableAttributes, UInt16Attributes,
    UInt32Attributes, UInt8Attributes,
};

pub trait AbstractVariableTrait {
    /// The full, unique name of the variable.
    fn name(&self) -> &str;
    /// A handle of the variable to efficiently identify the variable value in the model interface and for references within the modelDescription.xml
    fn value_reference(&self) -> u32;
    /// An optional description string describing the meaning of the variable.
    fn description(&self) -> Option<&str>;
    /// Enumeration that defines the causality of the variable.
    fn causality(&self) -> Causality;
    fn variability(&self) -> Variability;
    fn can_handle_multiple_set_per_time_instant(&self) -> bool;
}

pub trait ArrayableVariableTrait: AbstractVariableTrait {
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
        }
    };
}

macro_rules! impl_float_type {
    ($name:ident, $type:ty, $float_attr:ident) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        #[yaserde(root = "Float64")]
        pub struct $name {
            #[yaserde(flatten)]
            pub base_attr: RealBaseAttributes,
            #[yaserde(flatten)]
            pub attr: $float_attr,
            #[yaserde(flatten)]
            pub init_var: InitializableVariable,
            #[yaserde(attribute)]
            pub start: $type,
            #[yaserde(flatten)]
            pub real_var_attr: RealVariableAttributes,
        }

        impl_abstract_variable!($name);

        impl ArrayableVariableTrait for $name {
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
            pub fn start(&self) -> $type {
                self.start
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
    ($name:ident, $type:ty, $int_attr:ident) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        #[yaserde(root = "$name")]
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
    /// A data value that is constant during the simulation and is computed during initialization or when tunable parameters change.
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

#[derive(Clone, Copy, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum Variability {
    #[yaserde(rename = "constant")]
    Constant,
    #[yaserde(rename = "fixed")]
    Fixed,
    #[yaserde(rename = "tunable")]
    Tunable,
    #[yaserde(rename = "discrete")]
    Discrete,
    #[yaserde(rename = "continuous")]
    #[default]
    Continuous,
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

impl_float_type!(FmiFloat32, f32, Float32Attributes);
impl_float_type!(FmiFloat64, f64, Float64Attributes);

impl_integer_type!(FmiInt8, i8, Int8Attributes);
impl_integer_type!(FmiUInt8, u8, UInt8Attributes);
impl_integer_type!(FmiInt16, i16, Int16Attributes);
impl_integer_type!(FmiUInt16, u16, UInt16Attributes);
impl_integer_type!(FmiInt32, i32, Int32Attributes);
impl_integer_type!(FmiUInt32, u32, UInt32Attributes);

/*
#[derive(Debug, YaSerialize, YaDeserialize)]
#[yaserde(root = "ModelVariables")]
pub enum Fmi3Variable {
    #[yaserde(flatten, rename = "Float32")]
    Float32(FmiFloat32),
    #[yaserde(flatten, rename = "Float64")]
    Float64(FmiFloat64),
}

impl Default for Fmi3Variable {
    fn default() -> Self {
        Fmi3Variable::Float32(FmiFloat32::default())
    }
}
*/

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
    assert_eq!(var.start(), -9.81);
    assert_eq!(var.derivative(), Some(1));
    assert_eq!(var.description(), Some("Gravity acting on the ball"));
    assert_eq!(var.can_handle_multiple_set_per_time_instant(), false);
    assert_eq!(var.intermediate_update(), false);
}
