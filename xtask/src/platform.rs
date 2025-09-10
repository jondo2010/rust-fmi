/// Platform mappings from Rust targets to FMI platform identifiers
use std::collections::HashMap;

pub struct PlatformMapping {
    mappings: HashMap<&'static str, &'static str>,
}

impl PlatformMapping {
    pub fn new() -> Self {
        let mut mappings = HashMap::new();

        // Linux platforms
        mappings.insert("x86_64-unknown-linux-gnu", "x86_64-linux");
        mappings.insert("aarch64-unknown-linux-gnu", "aarch64-linux");
        mappings.insert("i686-unknown-linux-gnu", "x86-linux");

        // Windows platforms
        mappings.insert("x86_64-pc-windows-gnu", "x86_64-windows");
        mappings.insert("x86_64-pc-windows-msvc", "x86_64-windows");
        mappings.insert("i686-pc-windows-gnu", "x86-windows");
        mappings.insert("i686-pc-windows-msvc", "x86-windows");

        // macOS platforms
        mappings.insert("x86_64-apple-darwin", "x86_64-darwin");
        mappings.insert("aarch64-apple-darwin", "aarch64-darwin");

        Self { mappings }
    }

    pub fn get_fmi_platform(&self, rust_target: &str) -> Option<&str> {
        self.mappings.get(rust_target).copied()
    }

    pub fn get_library_extension(&self, rust_target: &str) -> &str {
        if rust_target.contains("windows") {
            "dll"
        } else if rust_target.contains("darwin") {
            "dylib"
        } else {
            "so"
        }
    }

    pub fn get_supported_targets(&self) -> Vec<&str> {
        self.mappings.keys().copied().collect()
    }
}
