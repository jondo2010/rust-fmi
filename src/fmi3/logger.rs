use super::{binding, FmiStatus};

/// Callback function for logging
pub(crate) unsafe extern "C" fn callback_log(
    _instance_environment: binding::fmi3InstanceEnvironment,
    instance_name: binding::fmi3String,
    status: binding::fmi3Status,
    category: binding::fmi3String,
    message: binding::fmi3String,
) {
    let instance_name = std::ffi::CStr::from_ptr(instance_name)
        .to_str()
        .unwrap_or("INVALID");
    let status = FmiStatus::from(status);
    let category = std::ffi::CStr::from_ptr(category)
        .to_str()
        .unwrap_or("INVALID");
    let message = std::ffi::CStr::from_ptr(message)
        .to_str()
        .unwrap_or("INVALID");

    println!(
        "instanceName: {}, status: {:?}, category: {}, message: {}",
        instance_name, status, category, message
    );

    log::log!(
        match status {
            FmiStatus::Ok => log::Level::Info,
            FmiStatus::Warning => log::Level::Warn,
            FmiStatus::Discard => log::Level::Warn,
            FmiStatus::Error => log::Level::Error,
            FmiStatus::Fatal => log::Level::Error,
        },
        "instanceName: {instance_name}, status: {status:?}, category: {category}, message: {message}",
    );
}
