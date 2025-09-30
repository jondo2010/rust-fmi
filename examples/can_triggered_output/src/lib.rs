#![deny(clippy::all)]

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{DefaultLoggingCategory, ModelContext, UserModel},
    FmuModel,
};

#[derive(FmuModel, Default, Debug)]
struct CanTriggeredOutput {
    #[variable(
        name = "CanChannel.Rx_Data",
        causality = Input,
        variability = Discrete,
        initial = Exact,
        max_size = 2048,
        clocks = [rx_clock],
        mime_type = "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"",
        start = ""
    )]
    rx_data: Vec<u8>,

    #[variable(
        name = "CanChannel.Tx_Data",
        causality = Output,
        variability = Discrete,
        initial = Calculated,
        max_size = 2048,
        clocks = [tx_clock],
        mime_type = "application/org.fmi-standard.fmi-ls-bus.can; version=\"1.0.0-beta.1\"",
    )]
    tx_data: Vec<u8>,

    #[variable(name="CanChannel.Rx_Clock", causality=Input, interval_variability=Triggered)]
    rx_clock: bool,

    #[variable(name="CanChannel.Tx_Clock", causality=Output, interval_variability=Triggered)]
    tx_clock: bool,

    #[variable(
        name = "org.fmi_standard.fmi_ls_bus.Can_BusNotifications",
        causality = StructuralParameter,
        variability = Fixed,
        initial = Exact,
        start = false
    )]
    can_bus_notifications: bool,
}

impl UserModel for CanTriggeredOutput {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(CanTriggeredOutput);
