use super::{Fmi3Status, binding};

/// Callback function for logging
pub(crate) unsafe extern "C" fn callback_log(
    _instance_environment: binding::fmi3InstanceEnvironment,
    status: binding::fmi3Status,
    category: binding::fmi3String,
    message: binding::fmi3String,
) {
    let status = Fmi3Status::from(status);
    let category = unsafe { std::ffi::CStr::from_ptr(category) }
        .to_str()
        .unwrap_or("INVALID");
    let message = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_str()
        .unwrap_or("INVALID");

    let level = match status.0 {
        binding::fmi3Status_fmi3OK => log::Level::Info,
        binding::fmi3Status_fmi3Warning => log::Level::Warn,
        binding::fmi3Status_fmi3Discard => log::Level::Warn,
        binding::fmi3Status_fmi3Error => log::Level::Error,
        binding::fmi3Status_fmi3Fatal => log::Level::Error,
        _ => unreachable!("Invalid status"),
    };

    log::logger().log(
        &log::Record::builder()
            .args(format_args!("{message}"))
            .level(level)
            .module_path(Some("fmu"))
            .target(category)
            .build(),
    );
}
