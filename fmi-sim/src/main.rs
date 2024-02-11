use fmi_sim::{options, simulate};
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    sensible_env_logger::try_init_timed!()?;

    let args = options::FmiCheckOptions::from_args();
    let output = simulate(args)?;

    println!(
        "Outputs:\n{}",
        arrow::util::pretty::pretty_format_batches(&[output]).unwrap()
    );

    Ok(())
}
