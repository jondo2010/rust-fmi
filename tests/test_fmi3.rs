use fmi::fmi3;
use yaserde_derive::YaDeserialize;

#[derive(Default, PartialEq, Debug, YaDeserialize)]
#[yaserde(rename = "fmiModelDescription")]
pub struct ModelDescriptionVersionOnly {
    /// Version of FMI that was used to generate the XML file.
    #[yaserde(attribute, rename = "fmiVersion")]
    pub fmi_version: String,
}

/// Check the FMI Version reported in the ModelDescription XML
fn check_meta_version(meta_content: &str) -> String {
    let meta: ModelDescriptionVersionOnly = yaserde::de::from_str(meta_content).unwrap();
    meta.fmi_version
}

#[test]
fn test_model_descr() {
    let meta_content = std::env::current_dir()
        .map(|path| path.join("tests/data/FMI3.xml"))
        .and_then(std::fs::read_to_string)
        .unwrap();

    match check_meta_version(&meta_content).as_str() {
        "3.0-beta.2" => {
            let m: fmi3::schema::FmiModelDescription =
                yaserde::de::from_str(&meta_content).unwrap();
            dbg!(&m.unit_definitions);

            let model = fmi3::model::ModelDescription::try_from(m).unwrap();

            dbg!(&model.model_variables);
        }
        "2.0" => {
            //let meta: fmi::model_descr::ModelDescription =
            //fmi::model_descr::from_reader(reader).unwrap();
        }
        _ => {}
    }
}
