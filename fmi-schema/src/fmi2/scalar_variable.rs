use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum Causality {
    #[yaserde(rename = "parameter")]
    Parameter,
    #[yaserde(rename = "calculatedParameter")]
    CalculatedParameter,
    #[yaserde(rename = "input")]
    Input,
    #[yaserde(rename = "output")]
    Output,
    #[yaserde(rename = "local")]
    Local,
    #[yaserde(rename = "independent")]
    Independent,
    #[default]
    #[yaserde(rename = "unknown")]
    Unknown,
}

/// Enumeration that defines the time dependency of the variable
#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
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
    Continuous,
    #[default]
    #[yaserde(rename = "unknown")]
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

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Real {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[yaserde(attribute, rename = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    /// max >= start >= min required
    #[yaserde(attribute)]
    pub start: f64,

    /// If present, this variable is the derivative of variable with ScalarVariable index
    /// "derivative".
    #[yaserde(attribute)]
    pub derivative: Option<u32>,

    /// Only for ModelExchange and if variable is a continuous-time state:
    /// If true, state can be reinitialized at an event by the FMU
    /// If false, state will never be reinitialized at an event by the FMU
    #[yaserde(attribute, rename = "reinit")]
    pub reinit: bool,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Integer {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[yaserde(attribute, rename = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    /// max >= start >= min required
    #[yaserde(attribute)]
    pub start: i32,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Boolean {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[yaserde(attribute, rename = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    #[yaserde(attribute)]
    pub start: bool,
}

#[derive(Clone, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum ScalarVariableElement {
    #[yaserde(flatten)]
    Real(Real),
    #[yaserde(flatten)]
    Integer(Integer),
    #[yaserde(flatten)]
    Boolean(Boolean),
    #[yaserde(flatten)]
    String,
    #[yaserde(flatten)]
    Enumeration,
}

impl Default for ScalarVariableElement {
    fn default() -> Self {
        Self::Real(Real::default())
    }
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct ScalarVariable {
    /// The full, unique name of the variable.
    #[yaserde(attribute)]
    pub name: String,

    /// A handle of the variable to efficiently identify the variable value in the model interface.
    #[yaserde(attribute, rename = "valueReference")]
    pub value_reference: u32,

    /// An optional description string describing the meaning of the variable.
    #[yaserde(attribute)]
    pub description: String,

    /// Enumeration that defines the causality of the variable.
    #[yaserde(attribute)]
    pub causality: Causality,

    /// Enumeration that defines the time dependency of the variable, in other words it defines the
    /// time instants when a variable can change its value.
    #[yaserde(attribute)]
    pub variability: Variability,

    /// Enumeration that defines how the variable is initialized. It is not allowed to provide a
    /// value for initial if `causality`=`Input` or `Independent`.
    #[yaserde(attribute)]
    pub initial: Initial,

    #[yaserde(flatten)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_variable() {
        let s = r#"
        <ScalarVariable
            name="inertia1.J"
            valueReference="1073741824"
            description="Moment of load inertia"
            causality="parameter"
            variability="fixed">
            <Real declaredType="Modelica.SIunits.Inertia" start="1"/>
        </ScalarVariable>
        "#;
        let sv: ScalarVariable = yaserde::de::from_str(s).unwrap();
        assert_eq!(sv.name, "inertia1.J");
        assert_eq!(sv.value_reference, 1073741824);
        assert_eq!(sv.description, "Moment of load inertia");
        assert_eq!(sv.causality, Causality::Parameter);
        assert_eq!(sv.variability, Variability::Fixed);
        assert_eq!(
            sv.elem,
            ScalarVariableElement::Real(Real {
                declared_type: Some("Modelica.SIunits.Inertia".to_string()),
                start: 1.0,
                derivative: None,
                reinit: false
            })
        );
    }
}
