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
            .allowlist_item("fmi2.*")
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
            .allowlist_item("fmi3.*")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Configure for Rust 2024 edition compatibility
            .wrap_unsafe_ops(true)
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join("fmi3_bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    #[cfg(feature = "ls-bus")]
    {
        let bindings = bindgen::Builder::default()
            .header("fmi-ls-bus/headers/fmi3LsBusCan.h")
            .header("fmi-ls-bus/headers/fmi3LsBusUtil.h")
            .clang_arg("-Ifmi-standard3/headers")
            .dynamic_link_require_all(false)
            .allowlist_item("fmi3LsBus.*")
            .allowlist_item("FMI3_LS_BUS.*")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            // Configure for Rust 2024 edition compatibility
            .wrap_unsafe_ops(true)
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(out_path.join("ls_bus_bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
