use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use super::fmi2;
use super::{FmiError, FmiResult};
use dlopen::wrapper::Container;
use log::trace;

const MODEL_DESCRIPTION: &str = "modelDescription.xml";

#[cfg(all(
    target_os = "windows",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
const FMI_PLATFORM: &str = "win64";
#[cfg(all(target_os = "windows", target_arch = "x86"))]
const FMI_PLATFORM: &str = "win32";
#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
const FMI_PLATFORM: &str = "linux64";
#[cfg(all(linux, target_arch = "x86"))]
const FMI_PLATFORM: &str = "linux32";
#[cfg(all(
    target_os = "macos",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
const FMI_PLATFORM: &str = "darwin64";
#[cfg(all(macos, target_arch = "x86"))]
const FMI_PLATFORM: &str = "darwin32";

fn construct_so_path(model_identifier: &str) -> FmiResult<PathBuf> {
    let fname = model_identifier.to_owned() + std::env::consts::DLL_SUFFIX;
    Ok(std::path::PathBuf::from("binaries")
        .join(FMI_PLATFORM)
        .join(fname))
}

fn extract_archive(archive: impl AsRef<Path>, outdir: impl AsRef<Path>) -> FmiResult<()> {
    let archive = archive.as_ref();
    let outdir = outdir.as_ref();
    trace!("Extracting {} into {}", archive.display(), outdir.display());
    let file = std::fs::File::open(archive)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = outdir.join(file.name());
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

pub struct Import {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    pub descr: Arc<fmi2::meta::ModelDescription>,
}

/// Implement Deserialize
#[cfg(feature = "deserialize")]
impl<'de> serde::Deserialize<'de> for Import {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Url,
            EnableFmiLogging,
        }

        struct ImportVisitor;
        impl<'de> serde::de::Visitor<'de> for ImportVisitor {
            type Value = Import;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Import")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Import, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut url = None;
                let mut enable_fmi_logging = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Url => {
                            if url.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url = Some(map.next_value()?);
                        }
                        Field::EnableFmiLogging => {
                            if enable_fmi_logging.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "enable_fmi_logging",
                                ));
                            }
                            enable_fmi_logging = Some(map.next_value()?);
                        }
                    }
                }
                let url = url.ok_or_else(|| serde::de::Error::missing_field("url"))?;
                let _enable_fmi_logging: bool = enable_fmi_logging
                    .ok_or_else(|| serde::de::Error::missing_field("enable_fmi_logging"))?;
                Import::new(url).map_err(serde::de::Error::custom)
            }
        }

        const FIELDS: &'static [&'static str] = &["url", "enable_fmi_logging"];
        deserializer.deserialize_struct("Import", FIELDS, ImportVisitor)
    }
}

impl std::fmt::Debug for Import {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Import {} {{FMI{}, {} variables}}",
            self.descr.model_name(),
            self.descr.fmi_version,
            self.descr.num_variables()
        )
    }
}

impl Import {
    /// Creates a new Import by extracting the FMU and parsing the modelDescription XML
    pub fn new(path: impl Into<std::path::PathBuf>) -> FmiResult<Import> {
        // First create a temp directory
        let temp_dir = tempfile::Builder::new().prefix("fmi-rs").tempdir()?;
        extract_archive(path.into(), temp_dir.path())?;
        //.context("extraction")?;

        // Open and parse the model description
        let descr_file_path = temp_dir.path().join(MODEL_DESCRIPTION);
        trace!("Parsing ModelDescription {:?}", descr_file_path);
        let descr_file = std::fs::File::open(descr_file_path)?;
        //.context(format!("{}", descr_file_path.as_path().display()))?;

        let descr: fmi2::meta::ModelDescription =
            fmi2::meta::from_reader(std::io::BufReader::new(descr_file))?;

        let cap_string = if descr.model_exchange.is_some() && descr.co_simulation.is_some() {
            "ME+CS".to_owned()
        } else if descr.model_exchange.is_some() {
            "ME".to_owned()
        } else if descr.co_simulation.is_some() {
            "CS".to_owned()
        } else {
            "".to_owned()
        };

        trace!(
            "Parsed modelDescription for \"{}\" ({})",
            descr.model_name(),
            cap_string
        );

        Ok(Import {
            dir: temp_dir,
            descr: Arc::new(descr),
        })
    }

    /// Create a ModelExchange API container if supported
    pub fn container_me(&self) -> FmiResult<Container<fmi2::Fmi2ME>> {
        let me = self
            .descr
            .model_exchange
            .as_ref()
            .ok_or(FmiError::UnsupportedFmuType(fmi2::fmi2Type::ModelExchange))?;
        trace!("Found ModelExchange model \"{}\"", me.model_identifier);

        let lib_path = self
            .dir
            .path()
            .join(construct_so_path(&me.model_identifier)?);
        trace!("Loading shared library {:?}", lib_path);

        unsafe { Container::load(lib_path) }.map_err(FmiError::from)
    }

    /// Create a CoSimulation API container if supported
    pub fn container_cs(&self) -> FmiResult<Container<fmi2::Fmi2CS>> {
        let cs = self
            .descr
            .co_simulation
            .as_ref()
            .ok_or(FmiError::UnsupportedFmuType(fmi2::fmi2Type::CoSimulation))?;
        trace!("Found CoSimulation model \"{}\"", cs.model_identifier);

        let lib_path = self
            .dir
            .path()
            .join(construct_so_path(&cs.model_identifier)?);
        trace!("Loading shared library {:?}", lib_path);

        unsafe { Container::load(lib_path) }.map_err(FmiError::from)
    }

    /// Return the path to the extracted FMU
    pub fn path(&self) -> &std::path::Path {
        self.dir.path()
    }

    /// Get a reference to the ModelDescription
    pub fn descr(&self) -> &fmi2::meta::ModelDescription {
        &self.descr
    }

    pub fn resource_url(&self) -> url::Url {
        url::Url::from_file_path(self.path().join("resources"))
            .expect("Error forming resource location URL")
    }
}

// TODO Make this work on other targets
#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_me() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();
        assert_eq!(import.descr.fmi_version, "2.0");

        let _me = import.container_me().unwrap();
    }

    #[test]
    fn test_import_cs() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();
        assert_eq!(import.descr.fmi_version, "2.0");

        let _cs = import.container_cs().unwrap();
    }
}
