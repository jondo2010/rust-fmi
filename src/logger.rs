use super::fmi;

/// This function gets called from logger.c
#[no_mangle]
extern "C" fn callback_log(
    _component_environment: fmi::fmi2ComponentEnvironment,
    instance_name: fmi::fmi2String,
    status: fmi::fmi2Status,
    category: fmi::fmi2String,
    message: fmi::fmi2String,
) {
    let instance_name = unsafe { std::ffi::CStr::from_ptr(instance_name) }
        .to_str()
        .unwrap();
    let level = match status {
        fmi::fmi2Status::OK => log::Level::Info,
        fmi::fmi2Status::Warning => log::Level::Warn,
        fmi::fmi2Status::Discard => log::Level::Trace,
        fmi::fmi2Status::Error => log::Level::Error,
        fmi::fmi2Status::Fatal => log::Level::Error,
        fmi::fmi2Status::Pending => log::Level::Info,
    };

    let _category = unsafe { std::ffi::CStr::from_ptr(category) }
        .to_str()
        .unwrap();

    let message = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_str()
        .unwrap();

    log::logger().log(
        &log::Record::builder()
            .args(format_args!("{}", message))
            .level(level)
            .module_path(Some("logger"))
            .file(Some("logger.rs"))
            .line(Some(0))
            .target(instance_name)
            .build(),
    );
}

#[link(name = "logger", kind = "static")]
extern "C" {
    /// This function is implemented in logger.c
    /// Note: This can be re-implemented in pure Rust once the `c_variadics` feature stabilizes.
    /// See: https://doc.rust-lang.org/beta/unstable-book/language-features/c-variadic.html
    pub(crate) fn callback_logger_handler(
        componentEnvironment: fmi::fmi2ComponentEnvironment,
        instanceName: fmi::fmi2String,
        status: fmi::fmi2Status,
        category: fmi::fmi2String,
        message: fmi::fmi2String,
        ...
    );
}
