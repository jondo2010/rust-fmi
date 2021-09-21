use super::fmi2;

/// This function gets called from logger.c
#[no_mangle]
extern "C" fn callback_log(
    _component_environment: fmi2::fmi2ComponentEnvironment,
    instance_name: fmi2::fmi2String,
    status: fmi2::fmi2Status,
    category: fmi2::fmi2String,
    message: fmi2::fmi2String,
) {
    let instance_name = unsafe { std::ffi::CStr::from_ptr(instance_name) }
        .to_str()
        .unwrap_or("NULL");
    let level = match status {
        fmi2::fmi2Status::OK => log::Level::Info,
        fmi2::fmi2Status::Warning => log::Level::Warn,
        fmi2::fmi2Status::Discard => log::Level::Trace,
        fmi2::fmi2Status::Error => log::Level::Error,
        fmi2::fmi2Status::Fatal => log::Level::Error,
        fmi2::fmi2Status::Pending => log::Level::Info,
    };

    let _category = unsafe { std::ffi::CStr::from_ptr(category) }
        .to_str()
        .unwrap_or("NULL");

    let message = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_str()
        .unwrap_or("NULL");

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
        componentEnvironment: fmi2::fmi2ComponentEnvironment,
        instanceName: fmi2::fmi2String,
        status: fmi2::fmi2Status,
        category: fmi2::fmi2String,
        message: fmi2::fmi2String,
        ...
    );
}
