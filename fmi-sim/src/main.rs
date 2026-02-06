use clap::Parser;

fn main() -> anyhow::Result<()> {
    let options = fmi_sim::options::FmiSimOptions::try_parse()?;

    // Initialize simple env_logger for now (temporary fallback)
    //env_logger::Builder::new()
    //    .filter_level(options.verbose.log_level_filter())
    //    .try_init()
    //    .unwrap_or_else(|_| {}); // Ignore if already initialized

    let _logger = flexi_logger::Logger::try_with_env()?.start()?;

    let stats = fmi_sim::simulate(&options)?;

    log::info!(
        "Simulation finished at t = {:.1} after {} steps and {} events.",
        stats.end_time,
        stats.num_steps,
        stats.num_events
    );

    Ok(())
}
