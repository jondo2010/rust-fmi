use std::{env, path::PathBuf};

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    #[cfg(feature = "fmi2")]
    {
        cc::Build::new()
            .file("src/fmi2/logger.c")
            .include("fmi-standard2/headers")
            .compile("liblogger.a");

        let bindings = bindgen::Builder::default()
            .header("fmi-standard2/headers/fmi2Functions.h")
            .dynamic_link_require_all(false)
            .dynamic_library_name("Fmi2Binding")
            .allowlist_function("fmi2.*")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Configure for Rust 2024 edition compatibility
            .wrap_unsafe_ops(true)
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join("fmi2_bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    #[cfg(feature = "fmi3")]
    {
        let bindings = bindgen::Builder::default()
            .header("fmi-standard3/headers/fmi3Functions.h")
            .dynamic_link_require_all(false)
            .dynamic_library_name("Fmi3Binding")
            .allowlist_function("fmi3.*")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Configure for Rust 2024 edition compatibility
            .wrap_unsafe_ops(true)
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join("fmi3_bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
