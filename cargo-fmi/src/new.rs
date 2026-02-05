use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
const DEFAULT_MODEL_NAME: &str = "Model";
const DEFAULT_DEP_VERSION: &str = "*";

pub struct NewArgs {
    pub path: PathBuf,
    pub name: Option<String>,
}

pub fn new_project(args: NewArgs) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("new").arg("--lib").arg(&args.path);
    if let Some(name) = &args.name {
        cmd.arg("--name").arg(name);
    }

    let status = cmd.status().context("Failed to run cargo new")?;
    if !status.success() {
        anyhow::bail!("cargo new failed with status {status}");
    }

    let manifest_path = args.path.join("Cargo.toml");
    let src_path = args.path.join("src").join("lib.rs");

    let _ = try_cargo_add(&args.path)?;
    let updated = update_manifest(&manifest_path, DEFAULT_DEP_VERSION)?;
    fs::write(&manifest_path, updated)
        .with_context(|| format!("Failed to write {}", manifest_path.display()))?;

    let lib_rs = render_lib_rs();
    fs::write(&src_path, lib_rs)
        .with_context(|| format!("Failed to write {}", src_path.display()))?;

    println!("Created FMI project at {}", args.path.display());

    Ok(())
}

fn try_cargo_add(project_root: &Path) -> Result<bool> {
    let status = Command::new("cargo")
        .current_dir(project_root)
        .arg("add")
        .arg("fmi")
        .arg("fmi-export")
        .status();

    match status {
        Ok(status) if status.success() => Ok(true),
        _ => Ok(false),
    }
}

fn update_manifest(
    manifest_path: &Path,
    dep_version: &str,
) -> Result<String> {
    let contents = fs::read_to_string(manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

    let mut updated = ensure_dependencies(&contents, dep_version);
    updated = ensure_lib_section(&updated);
    Ok(updated)
}

fn ensure_dependencies(contents: &str, dep_version: &str) -> String {
    let export_line = format!("fmi-export = \"{dep_version}\"\n");
    let fmi_line = format!("fmi = \"{dep_version}\"\n");

    let mut missing_lines = String::new();
    if !contents.contains("fmi-export =") {
        missing_lines.push_str(&export_line);
    }
    if !contents.contains("fmi =") {
        missing_lines.push_str(&fmi_line);
    }

    if missing_lines.is_empty() {
        return contents.to_string();
    }

    if let Some((head, tail)) = split_section(contents, "[dependencies]") {
        let mut merged = String::new();
        merged.push_str(head);
        merged.push_str("[dependencies]\n");
        merged.push_str(&missing_lines);
        merged.push_str(tail);
        return merged;
    }

    let mut updated = contents.to_string();
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str("\n[dependencies]\n");
    updated.push_str(&missing_lines);
    updated
}

fn ensure_lib_section(contents: &str) -> String {
    if contents.contains("[lib]") {
        if contents.contains("crate-type") {
            return contents.to_string();
        }

        let lines: Vec<&str> = contents.lines().collect();
        let mut out = String::new();
        let mut inserted = false;
        for (idx, line) in lines.iter().enumerate() {
            out.push_str(line);
            out.push('\n');
            if !inserted && line.trim() == "[lib]" {
                out.push_str("crate-type = [\"cdylib\"]\n");
                inserted = true;
            }
            if idx + 1 == lines.len() && !inserted {
                out.push_str("[lib]\ncrate-type = [\"cdylib\"]\n");
            }
        }
        return out;
    }

    let mut updated = contents.to_string();
    if !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str("\n[lib]\ncrate-type = [\"cdylib\"]\n");
    updated
}

fn split_section<'a>(contents: &'a str, header: &str) -> Option<(&'a str, &'a str)> {
    let pos = contents.find(header)?;
    let (before, after) = contents.split_at(pos);
    let after = &after[header.len()..];
    Some((before, after))
}

fn render_lib_rs() -> String {
    format!(
        "use fmi_export::FmuModel;\n\n#[derive(FmuModel, Default, Debug)]\npub struct {name} {{\n    #[variable(causality = Output, start = 0.0)]\n    y: f64,\n}}\n\nfmi_export::export_fmu!({name});\n",
        name = DEFAULT_MODEL_NAME
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_dependencies_adds_fmi_export_and_fmi() {
        let input = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n";
        let output = ensure_dependencies(input, "*");
        assert!(output.contains("[dependencies]\n"));
        assert!(output.contains("fmi-export = \"*\""));
        assert!(output.contains("fmi = \"*\""));
    }

    #[test]
    fn ensure_dependencies_does_not_duplicate() {
        let input = "[dependencies]\nfmi-export = \"*\"\nfmi = \"*\"\n";
        let output = ensure_dependencies(input, "*");
        let count = output.matches("fmi-export =").count();
        assert_eq!(count, 1);
        let fmi_count = output.matches("fmi =").count();
        assert_eq!(fmi_count, 1);
    }

    #[test]
    fn ensure_lib_section_inserts_crate_type() {
        let input = "[package]\nname = \"demo\"\n";
        let output = ensure_lib_section(input);
        assert!(output.contains("[lib]\ncrate-type = [\"cdylib\"]"));
    }

    #[test]
    fn ensure_lib_section_adds_crate_type_when_missing() {
        let input = "[lib]\nname = \"demo\"\n";
        let output = ensure_lib_section(input);
        assert!(output.contains("[lib]\ncrate-type = [\"cdylib\"]"));
    }
}
