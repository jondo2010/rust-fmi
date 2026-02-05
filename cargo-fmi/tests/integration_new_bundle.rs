use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

fn cargo_fmi_bin() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-fmi")
}

fn write_local_patches(project_root: &Path, repo_root: &Path) -> std::io::Result<()> {
    let cargo_dir = project_root.join(".cargo");
    std::fs::create_dir_all(&cargo_dir)?;
    let config_path = cargo_dir.join("config.toml");

    let config = format!(
        "[patch.crates-io]\n\
         fmi-export = {{ path = \"{root}/fmi-export\" }}\n\
         fmi-export-derive = {{ path = \"{root}/fmi-export-derive\" }}\n\
         fmi = {{ path = \"{root}/fmi\" }}\n\
         fmi-schema = {{ path = \"{root}/fmi-schema\" }}\n\
         fmi-sys = {{ path = \"{root}/fmi-sys\" }}\n",
        root = repo_root.display()
    );

    std::fs::write(config_path, config)
}

fn read_manifest(project_root: &Path) -> String {
    std::fs::read_to_string(project_root.join("Cargo.toml")).expect("read Cargo.toml")
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

    let output = Command::new(cargo_fmi_bin())
        .current_dir(&project_root)
        .env("CARGO_NET_OFFLINE", "true")
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
}
