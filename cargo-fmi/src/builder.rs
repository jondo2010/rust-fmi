use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use anyhow::Context;
use cargo_metadata::{Message, PackageId};

/// Build the library for the specified package and return paths to the built cdylib files.
///
/// If `target_triples` is provided, it can be used to specify custom target triples for cross-compilation.
/// If `None`, the default host target is used.
pub fn build_lib(
    package: &PackageId,
    target_triples: &Option<Vec<&'static platforms::Platform>>,
    release: bool,
) -> anyhow::Result<Vec<(Option<&'static platforms::Platform>, PathBuf)>> {
    let mut command = Command::new("cargo");

    command
        .args(&["build", "--lib", "--message-format=json-render-diagnostics"])
        .args(&["--package", &package.repr]);

    if let Some(platforms) = &target_triples {
        for platform in platforms {
            command.args(&["--target", platform.target_triple]);
        }
    } else {
        log::info!("No target triples specified, building for host platform");
    }

    if release {
        command.arg("--release");
    }

    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to start cargo build process")?;

    let reader = std::io::BufReader::new(child.stdout.take().unwrap());

    let mut dylib_paths = Vec::new();
    let mut build_success = false;

    for msg in Message::parse_stream(reader).filter_map(Result::ok) {
        match msg {
            Message::CompilerArtifact(artifact) if artifact.target.is_cdylib() => {
                // Heuristic to parse the target triple from the output filenames. When building for
                // specific target triples, the outputs will be placed in subdirectories named after
                // the target, for example "target/aarch64-apple-darwin/debug/libdahlquist.dylib".
                // When not building for specific targets, the output will be in
                // "target/debug/libdahlquist.dylib".

                // Find the actual dylib file (not debug symbols like .pdb on Windows)
                let path = artifact
                    .filenames
                    .iter()
                    .find(|p| {
                        let p = p.as_str();
                        // Keep only dynamic library files, exclude debug symbols
                        !(p.ends_with(".pdb") || p.ends_with(".dSYM"))
                    })
                    .map(|p| p.clone().into_std_path_buf())
                    .ok_or_else(|| {
                        anyhow::anyhow!("No output filenames found for cdylib target")
                    })?;

                let target = if let Some(_) = target_triples {
                    // Extract the target triple from the path components. It should be the 3rd from last.
                    Some(
                        path.components()
                            .nth_back(3)
                            .and_then(|target_triple| {
                                platforms::Platform::find(
                                    &target_triple.as_os_str().to_string_lossy(),
                                )
                            })
                            .ok_or_else(|| {
                                anyhow::anyhow!(
                                    "Could not determine target triple from path: {}",
                                    path.display()
                                )
                            })?,
                    )
                } else {
                    // No specific target triples were built, so this is the host platform
                    None
                };

                dylib_paths.push((target, path));
            }
            Message::BuildFinished(res) => {
                build_success = res.success;
                break; // No more messages after build finished
            }
            _ => { /* Ignore other messages */ }
        }
    }

    let output = child.wait().expect("Couldn't get cargo's exit status");

    if !dylib_paths.is_empty() {
        log::info!("Built cdylib artifacts: {:?}", dylib_paths);
    } else {
        log::warn!("No cdylib artifacts produced");
    }

    match (build_success, output.success()) {
        (true, true) => {
            log::info!("Build finished successfully");
            Ok(dylib_paths)
        }
        _ => {
            log::error!("Build failed");
            anyhow::bail!("Build failed");
        }
    }
}
