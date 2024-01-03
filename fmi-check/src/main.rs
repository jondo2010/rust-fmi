use fmi::{FmiImport, Import};
use fmi_schema::variable_counts::VariableCounts;
use structopt::StructOpt;

mod interpolation;
mod io;
mod options;
mod sim_fmi3;

trait Action {
    fn check(&self) -> anyhow::Result<()>;

    fn model_exchange(&self, options: options::Simulate) -> anyhow::Result<()>;

    fn co_simulation(&self, options: options::Simulate) -> anyhow::Result<()>;
}

impl Action for Import {
    fn check(&self) -> anyhow::Result<()> {
        let counts = match self {
            Import::Fmi2(fmi2) => fmi2.model_description().model_variables.model_counts(),
            Import::Fmi3(fmi3) => fmi3.model_description().model_variables.model_counts(),
        };

        // Pretty-print the counts
        println!("{counts}");

        Ok(())
    }

    fn model_exchange(&self, options: options::Simulate) -> anyhow::Result<()> {
        todo!();

        // match self {
        // Import::Fmi2(fmi2) => fmi2.model_exchange(options),
        // Import::Fmi3(fmi3) => fmi3.model_exchange(options),
        //}

        Ok(())
    }

    fn co_simulation(&self, options: options::Simulate) -> anyhow::Result<()> {
        match self {
            Import::Fmi2(import) => todo!(),
            Import::Fmi3(import) => sim_fmi3::co_simulation(import, &options),
        }
    }
}

fn main() -> anyhow::Result<()> {
    sensible_env_logger::try_init_timed!()?;

    let args = options::FmiCheckOptions::from_args();
    let import = fmi::Import::new(&args.model)?;

    match args.action {
        options::Action::Check => {
            import.check()?;
        }
        options::Action::ME(options) => {
            // let import = import.as_fmi2().unwrap();
            // import.model_exchange(options)?;
        }
        options::Action::CS(options) => {
            // let import = import.as_fmi2().unwrap();
            import.co_simulation(options)?;
        }
    }

    Ok(())
}
