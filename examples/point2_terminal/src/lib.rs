#![allow(unexpected_cfgs)]
#![deny(clippy::all)]
//! Point2 terminal example.
//!
//! This FMU exposes a single terminal named "position" with output variables
//! `x` and `y`. The terminal uses terminal_kind "Point2" so `fmi-sim` can map
//! it to a Foxglove Point2 JSON channel when using MCAP output.

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{Context, DefaultLoggingCategory, UserModel},
    FmuModel,
};

#[derive(FmuModel, Default, Debug)]
#[model(user_model = false, co_simulation = true, model_exchange = false)]
#[terminal(name = "position", matching_rule = "bus", terminal_kind = "Point2")]
struct Point2Terminal {
    #[variable(causality = Output, variability = Continuous, initial = Calculated)]
    x: f64,

    #[variable(causality = Output, variability = Continuous, initial = Calculated)]
    y: f64,
}

impl UserModel for Point2Terminal {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        let t = context.time();
        let radius = 1.0;
        self.x = radius * t.cos();
        self.y = radius * t.sin();
        Ok(Fmi3Res::OK)
    }
}

fmi_export::export_fmu!(Point2Terminal);
