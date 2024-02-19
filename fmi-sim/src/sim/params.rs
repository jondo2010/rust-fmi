use super::options::SimOptions;

pub struct SimParams {
    pub start_time: f64,
    pub stop_time: f64,
    pub output_interval: f64,
    pub tolerance: Option<f64>,

    /// Use event mode
    pub event_mode_used: bool,
    /// Support early-return in Co-Simulation.
    pub early_return_allowed: bool,
}

impl SimParams {
    /// Create a new `SimParams` from the given `SimOptions` and `DefaultExperiment`.
    ///
    /// Values from `SimOptions` take precedence over values from `DefaultExperiment`.
    pub fn new_from_options<DE: fmi_schema::traits::DefaultExperiment>(
        options: &SimOptions,
        default_experiment: &DE,
    ) -> anyhow::Result<Self> {
        let start_time = options
            .start_time
            .or(default_experiment.start_time())
            .unwrap_or(0.0);

        let stop_time = options
            .stop_time
            .or(default_experiment.stop_time())
            .unwrap_or(1.0);

        let output_interval = options
            .output_interval
            .or(default_experiment.step_size())
            .unwrap_or_else(|| (stop_time - start_time) / 500.0);

        if output_interval <= 0.0 {
            return Err(anyhow::anyhow!("`output_interval` must be positive."))?;
        }

        let tolerance = options.tolerance.or(default_experiment.tolerance());

        Ok(Self {
            start_time,
            stop_time,
            output_interval,
            tolerance,
            event_mode_used: options.event_mode_used,
            early_return_allowed: options.early_return_allowed,
        })
    }
}
