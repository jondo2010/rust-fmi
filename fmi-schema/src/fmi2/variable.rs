use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(transparent)]
pub struct ValueReference(
    //#[yaserde(deserialize_with = "t_from_str")]
    //pub(crate) binding::fmi2ValueReference,
);

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename_all = "camelCase")]
pub enum Causality {
    Parameter,
    CalculatedParameter,
    Input,
    Output,
    Local,
    Independent,
    #[default]
    Unknown,
}

/// Enumeration that defines the time dependency of the variable
#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename_all = "camelCase")]
pub enum Variability {
    Constant,
    Fixed,
    Tunable,
    Discrete,
    Continuous,
    #[default]
    Unknown,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename_all = "camelCase")]
pub enum Initial {
    #[default]
    Exact,
    Approx,
    Calculated,
}

#[derive(Clone, Default, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename_all = "camelCase")]
pub struct ScalarVariable {
    /// The full, unique name of the variable.
    pub name: String,

    //#[yaserde(deserialize_with = "t_from_str")]
    //pub value_reference: binding::fmi2ValueReference,
    /// A handle of the variable to efficiently identify the variable value in the model interface.
    pub value_reference: ValueReference,

    /// An optional description string describing the meaning of the variable.
    #[yaserde(default)]
    pub description: String,

    /// Enumeration that defines the causality of the variable.
    #[yaserde(default)]
    pub causality: Causality,

    /// Enumeration that defines the time dependency of the variable, in other words it defines the
    /// time instants when a variable can change its value.
    #[yaserde(default)]
    pub variability: Variability,

    /// Enumeration that defines how the variable is initialized. It is not allowed to provide a
    /// value for initial if `causality`=`Input` or `Independent`.
    #[yaserde(default)]
    pub initial: Initial,

    #[yaserde(rename = "$value")]
    pub elem: ScalarVariableElement,
}

impl ScalarVariable {
    pub fn is_continuous_input(&self) -> bool {
        matches!(
            (&self.elem, &self.causality),
            (ScalarVariableElement::Real { .. }, Causality::Input)
        )
    }
}

#[derive(Clone, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum ScalarVariableElement {
    #[yaserde(rename_all = "camelCase")]
    Real {
        declared_type: Option<String>,

        #[yaserde(default, deserialize_with = "t_from_str")]
        start: f64,

        #[yaserde(default, deserialize_with = "t_from_str")]
        relative_quantity: bool,

        //#[serde(default, deserialize_with = "deser_opt")]
        #[yaserde(default)]
        derivative: Option<u32>,
    },
    #[yaserde(rename_all = "camelCase")]
    Integer {
        #[yaserde(default)]
        declared_type: String,
        #[yaserde(default, deserialize_with = "t_from_str")]
        start: i64,
    },
    #[yaserde(rename_all = "camelCase")]
    Boolean {
        #[yaserde(default)]
        declared_type: String,
        #[yaserde(default, deserialize_with = "t_from_str")]
        start: bool,
    },
    #[yaserde(rename_all = "camelCase")]
    String {
        #[yaserde(default)]
        declared_type: String,
        start: String,
    },
    #[yaserde(rename_all = "camelCase")]
    Enumeration {
        #[yaserde(default)]
        declared_type: String,
        #[yaserde(default, deserialize_with = "t_from_str")]
        start: i64,
    },
}

impl Default for ScalarVariableElement {
    fn default() -> Self {
        ScalarVariableElement::Real {
            declared_type: None,
            start: 0.0,
            relative_quantity: false,
            derivative: None,
        }
    }
}
