use crate::fmi2 as binding;

/// This function gets called from logger.c
#[no_mangle]
extern "C" fn callback_log(
    _component_environment: binding::fmi2ComponentEnvironment,
    instance_name: binding::fmi2String,
    status: binding::fmi2Status,
    category: binding::fmi2String,
    message: binding::fmi2String,
) {
    let instance_name = unsafe { std::ffi::CStr::from_ptr(instance_name) }
        .to_str()
        .unwrap_or("NULL");

    let level = match status {
        binding::fmi2Status_fmi2OK => log::Level::Info,
        binding::fmi2Status_fmi2Warning => log::Level::Warn,
        binding::fmi2Status_fmi2Pending => unreachable!("Pending status is not allowed in logger"),
        binding::fmi2Status_fmi2Discard => log::Level::Trace,
        binding::fmi2Status_fmi2Error => log::Level::Error,
        binding::fmi2Status_fmi2Fatal => log::Level::Error,
        _ => unreachable!("Invalid status"),
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
            .target(instance_name)
            .build(),
    );
}

#[link(name = "logger", kind = "static")]
extern "C" {
    /// This function is implemented in logger.c
    /// Note: This can be re-implemented in pure Rust once the `c_variadics` feature stabilizes.
    /// See: https://doc.rust-lang.org/beta/unstable-book/language-features/c-variadic.html
    pub fn callback_logger_handler(
        componentEnvironment: binding::fmi2ComponentEnvironment,
        instanceName: binding::fmi2String,
        status: binding::fmi2Status,
        category: binding::fmi2String,
        message: binding::fmi2String,
        ...
    );
}
