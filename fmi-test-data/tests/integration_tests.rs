use fmi::traits::FmiImport;
use fmi_test_data::ReferenceFmus;

#[test]
fn test_multiple_fmu_access() {
    let mut reference_fmus = ReferenceFmus::new().unwrap();

    // Test accessing multiple different FMUs
    let bouncing_ball: fmi::fmi3::import::Fmi3Import =
        reference_fmus.get_reference_fmu("BouncingBall").unwrap();
    let dahlquist: fmi::fmi3::import::Fmi3Import =
        reference_fmus.get_reference_fmu("Dahlquist").unwrap();
    let van_der_pol: fmi::fmi3::import::Fmi3Import =
        reference_fmus.get_reference_fmu("VanDerPol").unwrap();

    assert_eq!(bouncing_ball.model_description().model_name, "BouncingBall");
    assert_eq!(dahlquist.model_description().model_name, "Dahlquist");
    assert_eq!(
        van_der_pol.model_description().model_name,
        "van der Pol oscillator"
    );
}

#[test]
fn test_fmi2_vs_fmi3_compatibility() {
    let mut reference_fmus = ReferenceFmus::new().unwrap();

    // Load the same FMU in both FMI 2.0 and 3.0 versions
    let fmu_v2: fmi::fmi2::import::Fmi2Import =
        reference_fmus.get_reference_fmu("BouncingBall").unwrap();
    let fmu_v3: fmi::fmi3::import::Fmi3Import =
        reference_fmus.get_reference_fmu("BouncingBall").unwrap();

    // Both should have the same model name but different FMI versions
    assert_eq!(fmu_v2.model_description().model_name, "BouncingBall");
    assert_eq!(fmu_v3.model_description().model_name, "BouncingBall");
    assert_eq!(fmu_v2.model_description().fmi_version, "2.0");
    assert_eq!(fmu_v3.model_description().fmi_version, "3.0");
}

#[test]
fn test_archive_consistency() {
    let mut reference_fmus = ReferenceFmus::new().unwrap();
    let available_fmus = reference_fmus.list_available_fmus().unwrap();

    // Should have at least some well-known FMUs
    assert!(
        available_fmus.len() >= 5,
        "Expected at least 5 FMUs, got {}",
        available_fmus.len()
    );

    // Test that we can actually load some of them
    for fmu_name in &available_fmus[..3.min(available_fmus.len())] {
        // Try to load as FMI 3.0 - should work for all Reference FMUs
        let result: Result<fmi::fmi3::import::Fmi3Import, _> =
            reference_fmus.get_reference_fmu(fmu_name);
        assert!(result.is_ok(), "Failed to load FMU: {}", fmu_name);
    }
}

#[test]
fn test_temp_file_extraction() {
    let mut reference_fmus = ReferenceFmus::new().unwrap();

    // Extract BouncingBall to a temporary file
    let temp_file = reference_fmus
        .extract_reference_fmu("BouncingBall", fmi::schema::MajorVersion::FMI3)
        .unwrap();

    // Verify the file exists and has reasonable size
    let metadata = std::fs::metadata(temp_file.path()).unwrap();
    assert!(
        metadata.len() > 1000,
        "FMU file seems too small: {} bytes",
        metadata.len()
    );
    assert!(
        metadata.len() < 50_000_000,
        "FMU file seems too large: {} bytes",
        metadata.len()
    );

    // Verify it's a valid zip file (FMU is a zip archive)
    let file = std::fs::File::open(temp_file.path()).unwrap();
    let zip_result = zip::ZipArchive::new(file);
    assert!(zip_result.is_ok(), "FMU is not a valid zip archive");
}

#[test]
fn test_version_info() {
    assert_eq!(ReferenceFmus::version(), "0.0.39");

    // Verify the version is reflected in the constants
    assert!(fmi_test_data::REF_ARCHIVE.contains("0.0.39"));
    assert!(fmi_test_data::REF_URL.contains("v0.0.39"));
}
