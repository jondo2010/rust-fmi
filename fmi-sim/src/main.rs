use clap::Parser;
use fmi_sim::{options, simulate};

fn main() -> anyhow::Result<()> {
    sensible_env_logger::try_init_timed!()?;

    let args = options::FmiCheckOptions::try_parse()?;
    let output = simulate(args)?;

    println!(
        "Outputs:\n{}",
        arrow::util::pretty::pretty_format_batches(&[output]).unwrap()
    );

    Ok(())
}
