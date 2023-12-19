use super::{binding, binding::fmi2ComponentEnvironment, FmiErr, FmiRes, FmiStatus};

/// This function gets called from logger.c
#[no_mangle]
extern "C" fn callback_log(
    _component_environment: fmi2ComponentEnvironment,
    instance_name: binding::fmi2String,
    status: binding::fmi2Status,
    category: binding::fmi2String,
    message: binding::fmi2String,
) {
    let instance_name = unsafe { std::ffi::CStr::from_ptr(instance_name) }
        .to_str()
        .unwrap_or("NULL");
    let status = Result::<FmiRes, FmiErr>::from(FmiStatus(status));
    let level = match status {
        Ok(FmiRes::OK) => log::Level::Info,
        Ok(FmiRes::Warning) => log::Level::Warn,
        Err(FmiErr::Discard) => log::Level::Trace,
        Err(FmiErr::Error) => log::Level::Error,
        Err(FmiErr::Fatal) => log::Level::Error,
        //Err(FmiErr::Pending) => log::Level::Info,
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
        componentEnvironment: fmi2ComponentEnvironment,
        instanceName: binding::fmi2String,
        status: binding::fmi2Status,
        category: binding::fmi2String,
        message: binding::fmi2String,
        ...
    );
}
