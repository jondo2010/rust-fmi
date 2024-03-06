use clap::Parser;

fn main() -> anyhow::Result<()> {
    sensible_env_logger::try_init_timed!()?;

    let options = fmi_sim::options::FmiSimOptions::try_parse()?;
    let outputs = fmi_sim::simulate(&options)?;

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
