use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{Float32Attributes, Float64Attributes, RealBaseAttributes, RealVariableAttributes};

pub trait AbstractVariableTrait {
    /// The full, unique name of the variable.
    fn name(&self) -> &str;
    /// A handle of the variable to efficiently identify the variable value in the model interface and for references within the modelDescription.xml
    fn value_reference(&self) -> u32;
    /// An optional description string describing the meaning of the variable.
    fn description(&self) -> Option<&str>;
    /// Enumeration that defines the causality of the variable.
    fn causality(&self) -> Causality;
    fn variability(&self) -> Option<Variability>;
    fn can_handle_multiple_set_per_time_instant(&self) -> bool;
}

pub trait ArrayableVariableTrait: AbstractVariableTrait {
    fn intermediate_update(&self) -> bool;
    fn previous(&self) -> u32;
}

pub trait TypedArrayableariableTrait: ArrayableVariableTrait {
    fn declared_type(&self) -> Option<&str>;
}

pub trait InitializableVariableTrait: TypedArrayableariableTrait {
    fn initial(&self) -> Option<Initial>;
}

macro_rules! impl_float_type {
    ($name:ident, $type:ty) => {
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
            fn variability(&self) -> Option<Variability> {
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

        impl TypedArrayableariableTrait for $name {
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

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize, Copy)]
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
    #[yaserde(rename = "structuralParameter")]
    StructuredParameter,
}

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize, Copy)]
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

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
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
    pub variability: Option<Variability>,
    #[yaserde(attribute, rename = "canHandleMultipleSetPerTimeInstant")]
    pub can_handle_multiple_set_per_time_instant: bool,
}

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
pub struct ArrayableVariable {
    #[yaserde(flatten)]
    pub abstract_var: AbstractVariable,
    #[yaserde(attribute, rename = "intermediateUpdate")]
    pub intermediate_update: bool,
    #[yaserde(attribute)]
    pub previous: u32,
}

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
pub struct TypedArrayableVariable {
    #[yaserde(flatten)]
    pub arrayable_var: ArrayableVariable,
    #[yaserde(attribute, rename = "declaredType")]
    pub declared_type: Option<String>,
}

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize, Copy)]
pub enum Initial {
    #[yaserde(rename = "exact")]
    #[default]
    Exact,
    #[yaserde(rename = "approx")]
    Approx,
    #[yaserde(rename = "calculated")]
    Calculated,
}

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
pub struct InitializableVariable {
    #[yaserde(flatten)]
    pub typed_arrayable_var: TypedArrayableVariable,
    #[yaserde(attribute)]
    pub initial: Option<Initial>,
}

#[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
pub struct FmiFloat32 {
    #[yaserde(flatten)]
    base_attr: RealBaseAttributes,
    #[yaserde(flatten)]
    attr: Float32Attributes,
    #[yaserde(flatten)]
    init_var: InitializableVariable,
    #[yaserde(attribute)]
    pub start: f32,
    #[yaserde(flatten)]
    real_var_attr: RealVariableAttributes,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiFloat64 {
    #[yaserde(flatten)]
    base_attr: RealBaseAttributes,
    #[yaserde(flatten)]
    attr: Float64Attributes,
    #[yaserde(flatten)]
    init_var: InitializableVariable,
    #[yaserde(attribute)]
    pub start: f64,
    #[yaserde(flatten)]
    real_var_attr: RealVariableAttributes,
}

impl_float_type!(FmiFloat32, f32);
impl_float_type!(FmiFloat64, f64);

#[test]
fn test_float64() {
    let s = r#"<Float64
        name="g"
        valueReference="5"
        causality="parameter"
        variability="fixed"
        initial="exact"
        declaredType="Acceleration"
        start="-9.81"
        derivative="1"
        description="Gravity acting on the ball"/>"#;
    let var = yaserde::de::from_str::<FmiFloat64>(s).unwrap();
    assert_eq!(
        FmiFloat64 {
            base_attr: RealBaseAttributes::default(),
            attr: Float64Attributes::default(),
            init_var: InitializableVariable {
                typed_arrayable_var: TypedArrayableVariable {
                    arrayable_var: ArrayableVariable {
                        abstract_var: AbstractVariable {
                            name: "g".to_owned(),
                            value_reference: 5,
                            description: Some("Gravity acting on the ball".to_owned()),
                            causality: Causality::Parameter,
                            variability: Some(Variability::Fixed),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    declared_type: Some("Acceleration".to_owned()),
                },
                initial: Some(Initial::Exact),
            },
            start: -9.81,
            real_var_attr: RealVariableAttributes {
                derivative: Some(1),
                ..Default::default()
            },
        },
        var
    );

    assert_eq!(var.name(), "g");
    assert_eq!(var.value_reference(), 5);
    assert_eq!(var.description(), Some("Gravity acting on the ball"));
    assert_eq!(var.causality(), Causality::Parameter);
    assert_eq!(var.variability(), Some(Variability::Fixed));
    assert_eq!(var.can_handle_multiple_set_per_time_instant(), false);
    assert_eq!(var.intermediate_update(), false);
    assert_eq!(var.start, -9.81);
}
