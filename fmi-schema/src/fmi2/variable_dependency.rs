use yaserde_derive::{YaDeserialize, YaSerialize};

/// Dependency of scalar Unknown from Knowns in Continuous-Time and Event Mode (ModelExchange), and
/// at Communication Points (CoSimulation): Unknown=f(Known_1, Known_2, ...).
/// The Knowns are "inputs", "continuous states" and "independent variable" (usually time)".
#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Fmi2VariableDependency {
    /// ScalarVariable index of Unknown
    #[yaserde(attribute = true)]
    pub index: u32,

    /// Defines the dependency of the Unknown (directly or indirectly via auxiliary variables) on
    /// the Knowns in Continuous-Time and Event Mode ([`super::ModelExchange`]) and at
    /// Communication Points ([`super::CoSimulation`]).
    ///
    /// If not present, it must be assumed that the Unknown depends on all Knowns. If present as
    /// empty list, the Unknown depends on none of the Knowns. Otherwise the Unknown depends on
    /// the Knowns defined by the given [`super::ScalarVariable`] indices. The indices are
    /// ordered according to size, starting with the smallest index.
    #[yaserde(attribute = true, rename = "dependencies")]
    pub dependencies: Vec<u32>,

    /// If not present, it must be assumed that the Unknown depends on the Knowns without a
    /// particular structure. Otherwise, the corresponding Known v enters the equation as:
    ///
    /// * [`DependenciesKind::Dependent`]:  no particular structure, f(v)
    /// * [`DependenciesKind::Constant`]:   constant factor, c*v (only for Real variablse)
    /// * [`DependenciesKind::Fixed`]:      fixed factor, p*v (only for Real variables)
    /// * [`DependenciesKind::Tunable`]:    tunable factor, p*v (only for Real variables)
    /// * [`DependenciesKind::Discrete`]:   discrete factor, d*v (only for Real variables)
    ///
    /// If [`Self::dependencies_kind`] is present, [`Self::dependencies`] must be present and must
    /// have the same number of list elements.
    #[yaserde(attribute = true, rename = "dependenciesKind")]
    pub dependencies_kind: Vec<DependenciesKind>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum DependenciesKind {
    #[yaserde(rename = "dependent")]
    #[default]
    Dependent,
    #[yaserde(rename = "constant")]
    Constant,
    #[yaserde(rename = "fixed")]
    Fixed,
    #[yaserde(rename = "tunable")]
    Tunable,
    #[yaserde(rename = "discrete")]
    Discrete,
}
