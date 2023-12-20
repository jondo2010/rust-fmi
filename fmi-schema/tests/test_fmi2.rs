//! Test FMI 2.0 schema by parsing the FMI2.xml file.

#[test]
#[cfg(feature = "fmi2")]
fn test_fmi2() {
    use fmi_schema::fmi2::FmiModelDescription;

    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI2.xml"))
        .unwrap();
    let file = std::fs::File::open(test_file).unwrap();
    let buf_reader = std::io::BufReader::new(file);
    let model: FmiModelDescription = yaserde::de::from_reader(buf_reader).unwrap();

    dbg!(model);
}
