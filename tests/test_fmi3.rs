use std::io::Read;

use fmi::fmi3;
use strong_xml::XmlRead;

#[derive(Debug)]
pub enum FmiMeta {
    None,
    Fmi1,
    Fmi2,
    Fmi3(fmi3::meta::ModelDescription),
}

fn fun_name(test_file: std::fs::File) -> FmiMeta {
    let mut reader = std::io::BufReader::new(test_file);
    let mut buf = String::new();
    reader.read_to_string(&mut buf).unwrap();
    let mut xml = strong_xml::XmlReader::new(&buf);
    xml.read_till_element_start("fmiModelDescription").unwrap();
    while let Ok(attr) = xml.find_attribute() {
        match attr {
            Some((attr, val)) if attr == "fmiVersion" && val == "3.0-beta.2" => {
                return FmiMeta::Fmi3(fmi3::meta::ModelDescription::from_str(&buf).unwrap());
            }
            Some((attr, val)) if attr == "fmiVersion" && val == "2.0" => {
                //let md: fmi::model_descr::ModelDescription = fmi::model_descr::from_reader(reader).unwrap();
            }
            _ => {}
        }
    }
    FmiMeta::None
}

#[test]
fn test_model_descr() {
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/data/FMI3.xml"))
        .and_then(std::fs::File::open)
        .unwrap();

    let meta = fun_name(test_file);
    dbg!(meta);
}
