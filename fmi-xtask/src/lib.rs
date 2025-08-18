use anyhow::{Context, Result};

/// The main xtask entry point function. See the readme for instructions on how to use this.
pub fn main() -> Result<()> {
    let args = std::env::args().skip(1);
    main_with_args("cargo xtask", args)
}

/// The main xtask entry point function, but with custom command line arguments. `args` should not
/// contain the command name, so you should always skip at least one argument from
/// `std::env::args()` before passing it to this function.
pub fn main_with_args(command_name: &str, args: impl IntoIterator<Item = String>) -> Result<()> {}

/// Change the current directory into the Cargo workspace's root.
///
/// This is using a heuristic to find the workspace root. It considers all ancestor directories of
/// either `CARGO_MANIFEST_DIR` or the current directory, and finds the leftmost one containing a
/// `Cargo.toml` file.
pub fn chdir_workspace_root() -> Result<()> {
    // This is either the directory of the xtask binary when using `nih_plug_xtask` normally, or any
    // random project when using it through `cargo nih-plug`.
    let project_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .context(
            "'$CARGO_MANIFEST_DIR' was not set and the current working directory could not be \
             found",
        )?;

    let workspace_root = project_dir
        .ancestors()
        .filter(|dir| dir.join("Cargo.toml").exists())
        // The ancestors are ordered starting from `project_dir` going up to the filesystem root. So
        // this is the leftmost matching ancestor.
        .last()
        .with_context(|| {
            format!(
                "Could not find a 'Cargo.toml' file in '{}' or any of its parent directories",
                project_dir.display()
            )
        })?;

    std::env::set_current_dir(workspace_root)
        .context("Could not change to workspace root directory")
}
