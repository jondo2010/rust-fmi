#![allow(unexpected_cfgs)]
#![deny(clippy::all)]

use fmi_export::{
    FmuModel,
    fmi3::{Binary, Clock, DefaultLoggingCategory, UserModel},
};

/// Simple terminal component that exposes RX/TX variables and clocks.
#[derive(FmuModel, Default, Debug)]
#[terminal(name = "SimpleTerminal", matching_rule = "bus")]
pub struct SimpleTerminal {
    #[variable(
        name = "Rx_Data",
        causality = Input,
        variability = Discrete,
        initial = Exact,
        max_size = 2048,
        clocks = [rx_clock],
        start = b""
    )]
    pub rx_data: Binary,

    #[variable(
        name = "Tx_Data",
        causality = Output,
        variability = Discrete,
        initial = Calculated,
        max_size = 2048,
    )]
    pub tx_data: Binary,

    #[variable(name = "Rx_Clock", causality = Input, interval_variability = Triggered)]
    pub rx_clock: Clock,

    #[variable(name = "Tx_Clock", causality = Output, interval_variability = Triggered)]
    pub tx_clock: Clock,
}

#[derive(FmuModel, Default, Debug)]
#[model(co_simulation = true, model_exchange = false, user_model = false)]
pub struct TerminalsSimple {
    #[child(prefix = "Powertrain")]
    #[terminal(name = "Powertrain")]
    terminal: SimpleTerminal,
}

impl UserModel for TerminalsSimple {
    type LoggingCategory = DefaultLoggingCategory;
}

fmi_export::export_fmu!(TerminalsSimple);
