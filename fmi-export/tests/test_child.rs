use fmi::schema::fmi3::{ModelStructure, ModelVariables};
use fmi_export::fmi3::Model;
use fmi_export::{
    FmuModel,
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
