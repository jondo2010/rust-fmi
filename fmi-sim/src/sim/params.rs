use fmi::FmiImport;

use super::options;

pub struct SimParams {
    pub start_time: f64,
    pub stop_time: f64,
    pub output_interval: f64,
    pub tolerance: Option<f64>,
}

impl SimParams {
    pub fn new(
        import: &fmi::fmi3::import::Fmi3Import,
        //import: &impl FmiModelDescription,
        options: &options::SimOptions,
    ) -> anyhow::Result<Self> {
        let md = import.model_description();

        let start_time = options
            .start_time
            .or(md.default_experiment.as_ref().and_then(|de| de.start_time))
            .unwrap_or(0.0);

        let stop_time = options
            .stop_time
            .or(md.default_experiment.as_ref().and_then(|de| de.stop_time))
            .unwrap_or(1.0);

        let output_interval = options
            .output_interval
            .or(md.default_experiment.as_ref().and_then(|de| de.step_size))
            .unwrap_or_else(|| (stop_time - start_time) / 500.0);

        if output_interval <= 0.0 {
            return Err(anyhow::anyhow!("`output_interval` must be positive."))?;
        }

        let tolerance = options
            .tolerance
            .or(md.default_experiment.as_ref().and_then(|de| de.tolerance));

        Ok(Self {
            start_time,
            stop_time,
            output_interval,
            tolerance,
        })
    }
}
