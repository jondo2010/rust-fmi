use super::{fmi, model_descr, Result};
use dlopen::wrapper::Container;
use failure::{format_err, ResultExt};
use log::trace;
use std::rc::Rc;

const MODEL_DESCRIPTION: &str = "modelDescription.xml";

fn construct_so_path(model_identifier: &str) -> std::path::PathBuf {
    let os = match std::env::consts::OS {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "win",
        &_ => "Unsupported",
    };
    let platform = match std::env::consts::ARCH {
        "x86_64" | "aarch64" | "mips64" | "powerpc64" | "sparc64" => "64",
        "x86" | "arm" | "mips" | "powerpc" | "s390x" => "32",
        &_ => "_Unknown",
    };
    let os_plat = os.to_owned() + platform;
    let fname = model_identifier.to_owned() + std::env::consts::DLL_SUFFIX;
    std::path::PathBuf::from("binaries")
        .join(os_plat)
        .join(fname)
}

fn extract_archive(archive: &std::path::Path, outdir: &std::path::Path) -> Result<()> {
    trace!(
        "Extracting {:?} into {:?}",
        archive.display(),
        outdir.display()
    );
    let file = std::fs::File::open(&archive).context(format!("{:?}", archive))?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = outdir.join(file.sanitized_name());

        if (&*file.name()).ends_with('/') {
            //trace!( "File {} extracted to \"{}\"", i, outpath.as_path().display());
            std::fs::create_dir_all(&outpath)?;
        } else {
            //trace!( "File {} extracted to \"{}\" ({} bytes)", i, outpath.as_path().display(), file.size());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p)?;
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
    descr: model_descr::ModelDescription,
}

/// Implement Deserialize
/*
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
                                return Err(serde::de::Error::duplicate_field("enable_fmi_logging"));
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
*/

impl std::fmt::Debug for Import {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Import {} {{FMI{}, {} variables}}",
            self.descr.model_name(),
            self.descr.fmi_version,
            self.descr().num_variables()
        )
    }
}

impl Import {
    /// Creates a new Import by extracting the FMU and parsing the modelDescription XML
    pub fn new(path: &std::path::Path) -> Result<Rc<Import>> {
        // First create a temp directory
        let temp_dir = tempfile::Builder::new().prefix("fmi-rs").tempdir()?;
        extract_archive(path, temp_dir.path()).context("extraction")?;

        // Open and parse the model description
        let descr_file_path = temp_dir.path().join(MODEL_DESCRIPTION);
        trace!("Parsing ModelDescription {:?}", descr_file_path);
        let descr_file = std::fs::File::open(descr_file_path)?;
        //.context(format!("{}", descr_file_path.as_path().display()))?;

        let descr: model_descr::ModelDescription =
            model_descr::from_reader(std::io::BufReader::new(descr_file))
                .map_err(failure::SyncFailure::new)?;

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

        Ok(Rc::new(Import {
            dir: temp_dir,
            descr: descr,
        }))
    }

    /// Create a ModelExchange API container if supported
    pub fn container_me(&self) -> Result<Container<fmi::Fmi2ME>> {
        let me = self
            .descr
            .model_exchange
            .as_ref()
            .ok_or(format_err!("ModelExchange not supported"))?;
        trace!("Found ModelExchange model \"{}\"", me.model_identifier);

        let lib_path = self
            .dir
            .path()
            .join(construct_so_path(&me.model_identifier));
        trace!("Loading shared library {:?}", lib_path);

        unsafe { Container::load(lib_path) }.map_err(failure::Error::from)
    }

    /// Create a CoSimulation API container if supported
    pub fn container_cs(&self) -> Result<Container<fmi::Fmi2CS>> {
        let cs = self
            .descr
            .co_simulation
            .as_ref()
            .ok_or(format_err!("CoSimulation not supported"))?;
        trace!("Found CoSimulation model \"{}\"", cs.model_identifier);

        let lib_path = self
            .dir
            .path()
            .join(construct_so_path(&cs.model_identifier));
        trace!("Loading shared library {:?}", lib_path);

        unsafe { Container::load(lib_path) }.map_err(failure::Error::from)
    }

    /// Return the path to the extracted FMU
    pub fn path(&self) -> &std::path::Path {
        self.dir.path()
    }

    /// Get a reference to the ModelDescription
    pub fn descr(&self) -> &model_descr::ModelDescription {
        &self.descr
    }

    pub fn resource_url(&self) -> Result<url::Url> {
        url::Url::from_file_path(self.path().join("resources"))
            .map_err(|_| format_err!("Error forming resource location URL"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //TODO Make this work on other targets
    #[cfg(target_os = "linux")]
    #[test]
    fn test_import_me() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();
        assert_eq!(import.descr().fmi_version, "2.0");

        let _me = import.container_me().unwrap();
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_import_cs() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();
        assert_eq!(import.descr().fmi_version, "2.0");

        let _cs = import.container_cs().unwrap();
    }
}
