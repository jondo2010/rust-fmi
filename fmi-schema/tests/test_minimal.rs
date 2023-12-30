use std::str::FromStr as _;

use semver::{BuildMetadata, Prerelease};

#[test]
fn test_minimal() -> Result<(), Box<dyn std::error::Error>> {
    let test_file = std::env::current_dir().map(|path| path.join("tests/FMI2.xml"))?;
    let data = std::fs::read_to_string(test_file)?;
    let md = fmi_schema::minimal::ModelDescription::from_str(&data)?;
    assert_eq!(md.version()?, semver::Version::new(2, 0, 0));
    assert_eq!(md.model_name, "BouncingBall");

    let test_file = std::env::current_dir().map(|path| path.join("tests/FMI3.xml"))?;
    let data = std::fs::read_to_string(test_file)?;
    let md = fmi_schema::minimal::ModelDescription::from_str(&data)?;
    assert_eq!(
        md.version()?,
        semver::Version {
            major: 3,
            minor: 0,
            patch: 0,
            pre: Prerelease::new("beta.2").unwrap(),
            build: BuildMetadata::EMPTY,
        }
    );
    assert_eq!(md.model_name, "BouncingBall");

    Ok(())
}
