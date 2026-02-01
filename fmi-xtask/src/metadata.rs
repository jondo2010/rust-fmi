use anyhow::Context;
use cargo_metadata::{CrateType, MetadataCommand, Package, Target};

use fmi::fmi3::{binding, schema};

pub struct MetadataBuilder {
    pub model_identifier: String,
    pub package: Package,
    pub target_dir: std::path::PathBuf,
}

impl MetadataBuilder {
    /// Create a new MetadataBuilder instance, optionally specifying a package name.
    /// If no package name is provided, the root package of the workspace is considered.
    pub fn new(pkg_name: Option<&str>) -> anyhow::Result<Self> {
        // Get the workspace metadata
        let workspace_root = std::env::current_dir()?;
        let workspace_metadata = MetadataCommand::new()
            .current_dir(&workspace_root)
            .exec()
            .context("Failed to execute cargo metadata")?;

        let package = if let Some(name) = pkg_name {
            workspace_metadata
                .packages
                .iter()
                .find(|p| p.name.as_str() == name)
                .with_context(|| format!("Package '{}' not found in workspace", name))?
        } else {
            workspace_metadata
                .root_package()
                .context("No root package found in workspace")?
        };

        let target = find_cdylib_target(package)?;

        log::info!(
            "Found target '{}' with crate types {:?} in package '{}'",
            target.name,
            target.crate_types,
            package.name
        );

        // Use target name as model identifier and fixed output directory
        Ok(Self {
            model_identifier: target.name.to_string(),
            package: package.clone(),
            target_dir: workspace_metadata.target_directory.into_std_path_buf(),
        })
    }
}

/// Find and return the first target of type `cdylib` in the given package.
fn find_cdylib_target(package: &Package) -> anyhow::Result<&Target> {
    let target = package
        .targets
        .iter()
        .find(|t| t.crate_types.iter().any(|ct| *ct == CrateType::CDyLib))
        .ok_or_else(|| {
            log::error!(
                "No cdylib target found in package '{}'. \
            Add 'crate-type = [\"cdylib\"]' to [lib] section in Cargo.toml",
                package.name
            );
            anyhow::anyhow!("No cdylib target found in package '{}'", package.name)
        })?;

    Ok(target)
}

/// Parse FMU metadata from package.metadata.fmu and convert to DefaultExperiment
pub fn parse_package_metadata(
    package: &Package,
) -> anyhow::Result<Option<schema::DefaultExperiment>> {
    // Check if the package has FMU metadata
    if let Some(fmu_metadata_value) = package.metadata.get("fmu") {
        log::debug!("Found FMU metadata in package: {:?}", fmu_metadata_value);

        // Look for default_experiment in the metadata
        if let Some(default_exp_value) = fmu_metadata_value.get("default_experiment") {
            log::debug!(
                "Parsing DefaultExperiment from metadata: {:?}",
                default_exp_value
            );

            // Parse directly into DefaultExperiment using serde
            let experiment: schema::DefaultExperiment =
                serde_json::from_value(default_exp_value.clone()).context(
                    "Failed to parse default_experiment metadata. Please check the format \
                    of your Cargo.toml [package.metadata.fmu.default_experiment] section.",
                )?;

            log::info!(
                "Successfully parsed DefaultExperiment: start_time={:?}, stop_time={:?}, \
                    tolerance={:?}, step_size={:?}",
                experiment.start_time,
                experiment.stop_time,
                experiment.tolerance,
                experiment.step_size
            );

            return Ok(Some(experiment));
        }
    }

    log::debug!("No FMU metadata found in package");
    Ok(None)
}

pub fn create_model_description(
    model_identifier: &str,
    package: &Package,
    model_data: crate::extractor::ModelData,
) -> anyhow::Result<schema::Fmi3ModelDescription> {
    // Parse FMU metadata for DefaultExperiment and other configurations
    let default_experiment =
        parse_package_metadata(package).context("Failed to parse FMU metadata from Cargo.toml")?;

    let fmi_version =
        unsafe { std::ffi::CStr::from_ptr(binding::fmi3Version.as_ptr() as _) }.to_string_lossy();

    // Build model description from package metadata
    Ok(schema::Fmi3ModelDescription {
        fmi_version: fmi_version.to_string(),
        model_name: package.name.as_str().to_string(),
        instantiation_token: model_data.instantiation_token,
        description: package.description.clone(),
        author: package.authors.first().map(|s| s.to_string()),
        version: Some(package.version.to_string()),
        copyright: package.license.clone(),
        license: package.license.clone(),
        generation_tool: Some("rust-fmi".to_string()),
        generation_date_and_time: Some(chrono::Utc::now().to_rfc3339()),
        // Set the extracted model variables and structure
        model_variables: model_data.model_variables,
        model_structure: model_data.model_structure,
        // Set the DefaultExperiment from metadata if present
        default_experiment,
        model_exchange: model_data
            .supports_model_exchange
            .then(|| schema::Fmi3ModelExchange {
                model_identifier: model_identifier.to_string(),
                can_get_and_set_fmu_state: Some(false),
                can_serialize_fmu_state: Some(false),
                ..Default::default()
            }),
        co_simulation: model_data
            .supports_co_simulation
            .then(|| schema::Fmi3CoSimulation {
                model_identifier: model_identifier.to_string(),
                can_handle_variable_communication_step_size: Some(true),
                has_event_mode: Some(false),
                ..Default::default()
            }),
        scheduled_execution: model_data.supports_scheduled_execution.then(|| {
            schema::Fmi3ScheduledExecution {
                model_identifier: model_identifier.to_string(),
                ..Default::default()
            }
        }),
        ..Default::default()
    })
}

/// Create a basic BuildDescription for the FMU
pub fn create_build_description(
    model_identifier: &str,
) -> anyhow::Result<schema::Fmi3BuildDescription> {
    let fmi_version =
        unsafe { std::ffi::CStr::from_ptr(binding::fmi3Version.as_ptr() as _) }.to_string_lossy();
    Ok(schema::Fmi3BuildDescription {
        fmi_version: fmi_version.to_string(),
        build_configurations: vec![schema::BuildConfiguration {
            model_identifier: model_identifier.to_string(),
            platform: Some("generic".to_string()),
            description: Some(format!(
                "Build configuration for {model_identifier} FMU generated from Rust"
            )),
            source_file_sets: vec![schema::SourceFileSet {
                name: Some("documentation".to_string()),
                language: Some("Markdown".to_string()),
                source_files: vec![schema::SourceFile {
                    name: "README.md".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }],
        ..Default::default()
    })
}
