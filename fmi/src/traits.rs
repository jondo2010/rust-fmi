use fmi_schema::traits::DefaultExperiment;

use crate::Error;

pub trait FmiImport: Sized {
    /// The raw parsed XML schema type
    type ModelDescription: DefaultExperiment;

    /// The raw FMI bindings type
    type Binding;

    /// The type of the value reference used by the FMI API.
    type ValueReference;

    /// Create a new FMI import from a directory containing the unzipped FMU
    fn new(dir: tempfile::TempDir, schema_xml: &str) -> Result<Self, Error>;

    /// Return the path to the extracted FMU
    fn archive_path(&self) -> &std::path::Path;

    /// Get the path to the shared library
    fn shared_lib_path(&self, model_identifier: &str) -> Result<std::path::PathBuf, Error>;

    /// Return the path to the resources directory
    fn resource_url(&self) -> url::Url {
        url::Url::from_file_path(self.archive_path().join("resources"))
            .expect("Error forming resource location URL")
    }

    /// Get a reference to the raw-schema model description
    fn model_description(&self) -> &Self::ModelDescription;

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error>;
}

/// Generic FMI instance trait
pub trait FmiInstance {
    type ModelDescription: DefaultExperiment;

    type Import: FmiImport<
        ModelDescription = Self::ModelDescription,
        ValueReference = Self::ValueReference,
    >;

    type ValueReference: Copy + From<u32> + Into<u32>;

    /// Get the name of the FMU
    fn name(&self) -> &str;

    /// Get the version of the FMU
    fn get_version(&self) -> &str;

    /// Get the model description of the FMU
    fn model_description(&self) -> &Self::ModelDescription;

    /// Get the number of values required to store the continuous states. Array dimensions are expanded.
    fn get_number_of_continuous_state_values(&mut self) -> usize;

    /// Get the number of values required to store the event indicators. Array dimensions are expanded.
    fn get_number_of_event_indicator_values(&mut self) -> usize;
}
