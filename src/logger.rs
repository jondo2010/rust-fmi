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
    use num_traits::cast::FromPrimitive;

    let instance_name = unsafe { std::ffi::CStr::from_ptr(instance_name) }
        .to_str()
        .unwrap();
    let status = match fmi::Status::from_u32(status) {
        Some(fmi::Status::OK) => log::Level::Info,
        Some(fmi::Status::Warning) => log::Level::Warn,
        Some(fmi::Status::Discard) => log::Level::Trace,
        Some(fmi::Status::Error) => log::Level::Error,
        Some(fmi::Status::Fatal) => log::Level::Error,
        Some(fmi::Status::Pending) => log::Level::Info,
        None => log::Level::Info,
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
            .level(status)
            .module_path(Some("logger"))
            .file(Some("logger.rs"))
            .line(Some(0))
            .target(instance_name)
            .build(),
    );
}

/// This function is implemented in logger.c
#[link(name = "logger", kind = "static")]
extern "C" {
    pub fn callback_logger_handler(
        componentEnvironment: fmi::fmi2ComponentEnvironment,
        instanceName: fmi::fmi2String,
        status: fmi::fmi2Status,
        category: fmi::fmi2String,
        message: fmi::fmi2String,
        ...
    );
}

#[test]
pub fn test_logger() {}
