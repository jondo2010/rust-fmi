use arrow::record_batch::RecordBatch;
use fmi::{FmiImport, Import};
use fmi_schema::variable_counts::VariableCounts;
use sim::options::SimOptions;

pub mod options;
pub mod sim;

trait Action {
    fn check(&self) -> anyhow::Result<()>;

    fn model_exchange(&self, options: SimOptions) -> anyhow::Result<RecordBatch>;

    fn co_simulation(&self, options: SimOptions) -> anyhow::Result<RecordBatch>;
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

    fn model_exchange(&self, options: SimOptions) -> anyhow::Result<RecordBatch> {
        match self {
            Import::Fmi2(_import) => todo!(),
            Import::Fmi3(import) => sim::fmi3_me::me_simulation(import, options),
        }
    }

    fn co_simulation(&self, options: SimOptions) -> anyhow::Result<RecordBatch> {
        match self {
            Import::Fmi2(_import) => todo!(),
            Import::Fmi3(import) => sim::fmi3_cs::co_simulation(import, options),
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
        options::Action::ME(options) => import.model_exchange(options),
        options::Action::CS(options) => import.co_simulation(options),
    }
}
