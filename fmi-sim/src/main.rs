use clap::Parser;

fn main() -> anyhow::Result<()> {
    let options = fmi_sim::options::FmiSimOptions::try_parse()?;

    // Initialize simple env_logger for now (temporary fallback)
    //env_logger::Builder::new()
    //    .filter_level(options.verbose.log_level_filter())
    //    .try_init()
    //    .unwrap_or_else(|_| {}); // Ignore if already initialized

    let _logger = flexi_logger::Logger::try_with_env()?.start()?;

    let (outputs, stats) = fmi_sim::simulate(&options)?;

    log::info!(
        "Simulation finished at t = {:.1} after {} steps.",
        stats.end_time,
        stats.num_steps
    );

    if let Some(output_file) = options.output_file {
        let file = std::fs::File::create(output_file).unwrap();
        arrow::csv::writer::WriterBuilder::new()
            .with_delimiter(options.separator as _)
            .with_header(true)
            .build(file)
            .write(&outputs)?;
    } else {
        println!(
            "Outputs:\n{}",
            arrow::util::pretty::pretty_format_batches(&[outputs]).unwrap()
        );
    }

    Ok(())
}
