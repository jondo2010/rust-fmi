use arrow::record_batch::RecordBatch;
use fmi::{FmiImport, Import};
use fmi_schema::variable_counts::VariableCounts;

mod io;
pub mod options;
mod sim_fmi3;

trait Action {
    fn check(&self) -> anyhow::Result<()>;

    fn model_exchange(&self, options: options::Simulate) -> anyhow::Result<RecordBatch>;

    fn co_simulation(&self, options: options::Simulate) -> anyhow::Result<RecordBatch>;
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

    fn model_exchange(&self, options: options::Simulate) -> anyhow::Result<RecordBatch> {
        todo!();

        // match self {
        // Import::Fmi2(fmi2) => fmi2.model_exchange(options),
        // Import::Fmi3(fmi3) => fmi3.model_exchange(options),
        //}
    }

    fn co_simulation(&self, options: options::Simulate) -> anyhow::Result<RecordBatch> {
        match self {
            Import::Fmi2(import) => todo!(),
            Import::Fmi3(import) => sim_fmi3::co_simulation(import, &options),
        }
    }
}

pub fn simulate(args: options::FmiCheckOptions) -> anyhow::Result<RecordBatch> {
    let import = fmi::Import::new(&args.model)?;
    match args.action {
        options::Action::Check => {
            import.check()?;
            todo!();
        }
        options::Action::ME(options) => {
            // let import = import.as_fmi2().unwrap();
            // import.model_exchange(options)?;
            todo!();
        }
        options::Action::CS(options) => {
            // let import = import.as_fmi2().unwrap();
            import.co_simulation(options)
        }
    }
}
