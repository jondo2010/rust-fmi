use fmi::schema::fmi3::{ModelStructure, ModelVariables};
use fmi_export::fmi3::{Model, UserModelCS, UserModelME};
use fmi_export::{
    FmuModel,
    fmi3::{DefaultLoggingCategory, UserModel},
};

#[derive(FmuModel, Default, Debug)]
#[model(model_exchange = true)]
struct TestModel {
    #[variable(causality = Output, variability = Continuous, start = 0.0, initial = Exact)]
    y: f64,
}

#[derive(FmuModel, Debug, Default)]
struct TestModelParent {
    #[child]
    child: TestModel,
}

impl UserModel for TestModelParent {
    type LoggingCategory = DefaultLoggingCategory;
}

impl UserModelME for TestModelParent {}
impl UserModelCS for TestModelParent {
    fn do_step(
        &mut self,
        context: &mut dyn fmi_export::fmi3::Context<Self>,
        current_communication_point: f64,
        communication_step_size: f64,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> Result<fmi_export::fmi3::CSDoStepResult, fmi::fmi3::Fmi3Error> {
        todo!()
    }
}

fmi_export::export_fmu!(TestModelParent);

#[test]
fn child_variables_are_prefixed() {
    let mut variables = ModelVariables::default();
    let mut model_structure = ModelStructure::default();

    let count = TestModelParent::build_metadata(&mut variables, &mut model_structure, 0, None);

    let names: Vec<_> = variables
        .iter_abstract()
        .map(|v| v.name().to_string())
        .collect();

    assert_eq!(count, 1);
    assert_eq!(names, vec!["child.y".to_string()]);
}
