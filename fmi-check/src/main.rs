use structopt::StructOpt;

mod input;
mod interpolation;
mod options;
mod sim_fmi3;

fn main() -> anyhow::Result<()> {
    sensible_env_logger::init!();

    let args = options::FmiCheckOptions::from_args_safe()?;

    let import = fmi::import::Import::new(args.model)?;

    match import {
        fmi::Import::Fmi2(fmi2) => todo!(),
        fmi::Import::Fmi3(fmi3) => {}
    }

    Ok(())
}
