use std::{fmt::Display, str::FromStr};

/// Enumeration that defines the causality of the variable.
#[derive(Clone, Default, PartialEq, Debug)]
pub enum Causality {
    Parameter,
    CalculatedParameter,
    Input,
    Output,
    #[default]
    Local,
    Independent,
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
            _ => Err(format!("Invalid Causality: {}", s)),
        }
    }
}

impl Display for Causality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Causality::Parameter => "parameter",
            Causality::CalculatedParameter => "calculatedParameter",
            Causality::Input => "input",
            Causality::Output => "output",
            Causality::Local => "local",
            Causality::Independent => "independent",
        };
        write!(f, "{}", s)
    }
}

/// Enumeration that defines the time dependency of the variable, in other words it defines the time instants when a variable can change its value.
///
/// The default is [`Variability::Continuous`].
#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub enum Variability {
    /// The value of the variable never changes.
    Constant,
    /// The value of the variable is fixed after initialization, in other words after `exit_initialization_mode()` was called the variable value does not change anymore.
    Fixed,
    /// The value of the variable is constant between external events (ModelExchange) and between Communication Points (CoSimulation) due to changing variables with causality = "parameter" or "input" and variability = "tunable".
    Tunable,
    /// * ModelExchange: The value of the variable is constant between external and internal events (= time, state, step events defined implicitly in the FMU).
    /// * CoSimulation: By convention, the variable is from a "real" sampled data system and its value is only changed at Communication Points (also inside the slave).
    Discrete,
    /// Only a variable of type = "Real" can be "continuous".
    /// * ModelExchange: No restrictions on value changes.
    /// * CoSimulation: By convention, the variable is from a differential
    #[default]
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
        let s = match self {
            Variability::Constant => "constant",
            Variability::Fixed => "fixed",
            Variability::Tunable => "tunable",
            Variability::Discrete => "discrete",
            Variability::Continuous => "continuous",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
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
        let s = match self {
            Initial::Exact => "exact",
            Initial::Approx => "approx",
            Initial::Calculated => "calculated",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Real")]
pub struct Real {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[xml(attr = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    /// max >= start >= min required
    #[xml(attr = "start")]
    pub start: Option<f64>,

    /// If present, this variable is the derivative of variable with ScalarVariable index
    /// "derivative".
    #[xml(attr = "derivative")]
    pub derivative: Option<u32>,

    /// Only for ModelExchange and if variable is a continuous-time state:
    /// If true, state can be reinitialized at an event by the FMU
    /// If false, state will never be reinitialized at an event by the FMU
    #[xml(attr = "reinit")]
    pub reinit: Option<bool>,
}

#[derive(Clone, Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Integer")]
pub struct Integer {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[xml(attr = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    /// max >= start >= min required
    #[xml(attr = "start")]
    pub start: Option<i32>,
}

#[derive(Clone, Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Boolean")]
pub struct Boolean {
    /// If present, name of type defined with TypeDefinitions / SimpleType providing defaults.
    #[xml(attr = "declaredType")]
    pub declared_type: Option<String>,

    /// Value before initialization, if initial=exact or approx.
    #[xml(attr = "start")]
    pub start: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
pub enum ScalarVariableElement {
    #[xml(tag = "Real")]
    Real(Real),
    #[xml(tag = "Integer")]
    Integer(Integer),
    #[xml(tag = "Boolean")]
    Boolean(Boolean),
    #[xml(tag = "String")]
    String,
    #[xml(tag = "Enumeration")]
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

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ScalarVariable", strict(unknown_attribute, unknown_element))]
pub struct ScalarVariable {
    /// The full, unique name of the variable.
    #[xml(attr = "name")]
    pub name: String,

    /// A handle of the variable to efficiently identify the variable value in the model interface.
    #[xml(attr = "valueReference")]
    pub value_reference: u32,

    /// An optional description string describing the meaning of the variable.
    #[xml(attr = "description")]
    pub description: Option<String>,

    /// Enumeration that defines the causality of the variable.
    #[xml(attr = "causality", default)]
    pub causality: Causality,

    /// Enumeration that defines the time dependency of the variable, in other words it defines the
    /// time instants when a variable can change its value.
    #[xml(attr = "variability")]
    pub variability: Option<Variability>,

    /// Enumeration that defines how the variable is initialized. It is not allowed to provide a
    /// value for initial if `causality`=`Input` or `Independent`.
    #[xml(attr = "initial")]
    pub initial: Option<Initial>,

    #[xml(
        child = "Real",
        child = "Integer",
        child = "Boolean",
        child = "String",
        child = "Enumeration"
    )]
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
    use hard_xml::XmlRead;

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
        let sv = ScalarVariable::from_str(s).unwrap();
        assert_eq!(sv.name, "inertia1.J");
        assert_eq!(sv.value_reference, 1073741824);
        assert_eq!(sv.description, Some("Moment of load inertia".into()));
        assert_eq!(sv.causality, Causality::Parameter);
        assert_eq!(sv.variability, Some(Variability::Fixed));
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
