use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
/// Dependency of scalar Unknown from Knowns in Continuous-Time and Event Mode (ModelExchange), and at Communication
/// Points (CoSimulation): Unknown=f(Known_1, Known_2, ...).
/// The Knowns are "inputs", "continuous states" and "independent variable" (usually time)".
pub struct Unknown {
    #[yaserde(attribute)]
    /// ScalarVariable index of Unknown
    pub index: u32,

    #[yaserde(attribute)]
    #[cfg(feature = "disabled")]
    /// Defines the dependency of the Unknown (directly or indirectly via auxiliary variables) on the Knowns in
    /// Continuous-Time and Event Mode (ModelExchange) and at Communication Points (CoSimulation). If not present, it
    /// must be assumed that the Unknown depends on all Knowns. If present as empty list, the Unknown depends on none of
    /// the Knowns. Otherwise the Unknown depends on the Knowns defined by the given ScalarVariable indices. The indices
    /// are ordered according to size, starting with the smallest index.
    pub dependencies: Vec<u32>,
}
