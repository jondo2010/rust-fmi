use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::default_wrapper;

/// Enumeration that defines the causality of the variable.
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
    #[default]
    #[yaserde(rename = "local")]
    Local,
    #[yaserde(rename = "independent")]
    Independent,
}

/// Enumeration that defines the time dependency of the variable, in other words it defines the time instants when a variable can change its value.
///
/// The default is [`Variability::Continuous`].
#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum Variability {
    /// The value of the variable never changes.
    #[yaserde(rename = "constant")]
    Constant,
    /// The value of the variable is fixed after initialization, in other words after `exit_initialization_mode()` was called the variable value does not change anymore.
    #[yaserde(rename = "fixed")]
    Fixed,
    /// The value of the variable is constant between external events (ModelExchange) and between Communication Points (CoSimulation) due to changing variables with causality = "parameter" or "input" and variability = "tunable".
    #[yaserde(rename = "tunable")]
    Tunable,
    /// * ModelExchange: The value of the variable is constant between external and internal events (= time, state, step events defined implicitly in the FMU).
    /// * CoSimulation: By convention, the variable is from a “real” sampled data system and its value is only changed at Communication Points (also inside the slave).
    #[yaserde(rename = "discrete")]
    Discrete,
    /// Only a variable of type = "Real" can be "continuous".
    /// * ModelExchange: No restrictions on value changes.
    /// * CoSimulation: By convention, the variable is from a differential
    #[default]
    #[yaserde(rename = "continuous")]
    Continuous,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum Initial {
    #[default]
    #[yaserde(rename = "exact")]
    Exact,
    #[yaserde(rename = "approx")]
    Approx,
    #[yaserde(rename = "calculated")]
    Calculated,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Real {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[yaserde(attribute = true, rename = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    /// max >= start >= min required
    #[yaserde(attribute = true)]
    pub start: Option<f64>,

    /// If present, this variable is the derivative of variable with ScalarVariable index
    /// "derivative".
    #[yaserde(attribute = true)]
    pub derivative: Option<u32>,

    /// Only for ModelExchange and if variable is a continuous-time state:
    /// If true, state can be reinitialized at an event by the FMU
    /// If false, state will never be reinitialized at an event by the FMU
    #[yaserde(attribute = true, rename = "reinit")]
    pub reinit: Option<bool>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Integer {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[yaserde(attribute = true, rename = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    /// max >= start >= min required
    #[yaserde(attribute = true)]
    pub start: Option<i32>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Boolean {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[yaserde(attribute = true, rename = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    #[yaserde(attribute = true)]
    pub start: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum ScalarVariableElement {
    #[yaserde(flatten = true)]
    Real(Real),
    #[yaserde(flatten = true)]
    Integer(Integer),
    #[yaserde(flatten = true)]
    Boolean(Boolean),
    #[yaserde(flatten = true)]
    String,
    #[yaserde(flatten = true)]
    Enumeration,
}

impl Default for ScalarVariableElement {
    fn default() -> Self {
        Self::Real(Real::default())
    }
}

#[cfg(feature = "arrow")]
impl ScalarVariableElement {
    pub fn data_type(&self) -> arrow::datatypes::DataType {
        match self {
            ScalarVariableElement::Real(_) => arrow::datatypes::DataType::Float64,
            ScalarVariableElement::Integer(_) => arrow::datatypes::DataType::Int32,
            ScalarVariableElement::Boolean(_) => arrow::datatypes::DataType::Boolean,
            ScalarVariableElement::String => arrow::datatypes::DataType::Utf8,
            ScalarVariableElement::Enumeration => arrow::datatypes::DataType::Int32,
        }
    }
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct ScalarVariable {
    /// The full, unique name of the variable.
    #[yaserde(attribute = true)]
    pub name: String,

    /// A handle of the variable to efficiently identify the variable value in the model interface.
    #[yaserde(attribute = true, rename = "valueReference")]
    pub value_reference: u32,

    /// An optional description string describing the meaning of the variable.
    #[yaserde(attribute = true)]
    pub description: Option<String>,

    /// Enumeration that defines the causality of the variable.
    #[yaserde(attribute = true, default = "default_wrapper")]
    pub causality: Causality,

    /// Enumeration that defines the time dependency of the variable, in other words it defines the
    /// time instants when a variable can change its value.
    #[yaserde(attribute = true, default = "default_wrapper")]
    pub variability: Variability,

    /// Enumeration that defines how the variable is initialized. It is not allowed to provide a
    /// value for initial if `causality`=`Input` or `Independent`.
    #[yaserde(attribute = true)]
    pub initial: Option<Initial>,

    #[yaserde(flatten = true)]
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
        assert_eq!(sv.description, Some("Moment of load inertia".into()));
        assert_eq!(sv.causality, Causality::Parameter);
        assert_eq!(sv.variability, Variability::Fixed);
        assert_eq!(
            sv.elem,
            ScalarVariableElement::Real(Real {
                declared_type: Some("Modelica.SIunits.Inertia".to_string()),
                start: Some(1.0),
                derivative: None,
                reinit: None
            })
        );
    }
}
