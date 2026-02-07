use std::path::Path;
use std::process::Command;

use tempfile::TempDir;
use zip::ZipArchive;

fn cargo_fmi_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-fmi")
}

fn write_local_patches(project_root: &Path, repo_root: &Path) -> std::io::Result<()> {
    let cargo_dir = project_root.join(".cargo");
    std::fs::create_dir_all(&cargo_dir)?;
    let config_path = cargo_dir.join("config.toml");

    let root = repo_root.to_string_lossy().replace('\\', "/");
    let config = format!(
        "[patch.crates-io]\n\
         fmi-export = {{ path = \"{root}/fmi-export\" }}\n\
         fmi-export-derive = {{ path = \"{root}/fmi-export-derive\" }}\n\
         fmi = {{ path = \"{root}/fmi\" }}\n\
         fmi-schema = {{ path = \"{root}/fmi-schema\" }}\n\
         fmi-sys = {{ path = \"{root}/fmi-sys\" }}\n\
         fmi-ls-bus = {{ path = \"{root}/fmi-ls-bus\" }}\n",
        root = root
    );

    std::fs::write(config_path, config)
}

fn read_manifest(project_root: &Path) -> String {
    std::fs::read_to_string(project_root.join("Cargo.toml")).expect("read Cargo.toml")
}

fn add_dependency(project_root: &Path, dep_line: &str) {
    let manifest_path = project_root.join("Cargo.toml");
    let mut manifest = read_manifest(project_root);
    if manifest.contains(dep_line) {
        return;
    }
    if let Some((head, tail)) = manifest.split_once("[dependencies]\n") {
        manifest = format!("{head}[dependencies]\n{dep_line}{tail}");
    } else {
        manifest.push_str("\n[dependencies]\n");
        manifest.push_str(dep_line);
    }
    std::fs::write(manifest_path, manifest).expect("write Cargo.toml");
}

fn write_demo_lib(project_root: &Path) {
    let lib_rs = r#"use fmi_export::FmuModel;
use fmi_ls_bus::can::CanBus;

#[derive(FmuModel, Default, Debug)]
pub struct Model {
    #[child(prefix = "Powertrain")]
    #[terminal(name = "Powertrain")]
    bus: CanBus,
}

fmi_export::export_fmu!(Model);
"#;
    std::fs::write(project_root.join("src").join("lib.rs"), lib_rs).expect("write demo lib.rs");
}

#[test]
fn cargo_fmi_new_and_bundle() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let temp = TempDir::new().expect("temp dir");
    let project_root = temp.path().join("demo_fmu");

    let output = Command::new(cargo_fmi_bin())
        .arg("new")
        .arg(&project_root)
        .arg("--name")
        .arg("demo_fmu")
        .output()
        .expect("run cargo fmi new");
    assert!(
        output.status.success(),
        "cargo fmi new failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    write_local_patches(&project_root, repo_root).expect("write local patches");
    add_dependency(
        &project_root,
        "fmi-ls-bus = { version = \"*\", features = [\"fmi-export\"] }\n",
    );
    write_demo_lib(&project_root);

    let output = Command::new(cargo_fmi_bin())
        .current_dir(&project_root)
        .arg("bundle")
        .output()
        .expect("run cargo fmi bundle");
    assert!(
        output.status.success(),
        "cargo fmi bundle failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest = read_manifest(&project_root);
    assert!(manifest.contains("[lib]\ncrate-type = [\"cdylib\"]"));

    let fmu_path = project_root.join("target").join("fmu").join("demo_fmu.fmu");
    assert!(fmu_path.exists());

    let file = std::fs::File::open(&fmu_path).expect("open fmu");
    let mut archive = ZipArchive::new(file).expect("open zip");
    let mut entry = archive
        .by_name("resources/terminalsAndIcons/terminalsAndIcons.xml")
        .expect("terminalsAndIcons.xml present");
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut entry, &mut xml).expect("read terminals xml");
    assert!(xml.contains("fmiTerminalsAndIcons"));
    assert!(xml.contains("Powertrain"));
}
