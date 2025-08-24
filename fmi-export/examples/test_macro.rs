// Einfacher Test um zu sehen, was das Macro generiert
use fmi_export::FmuModel;

#[derive(Debug, Default, FmuModel)]
#[model(ModelExchange)]
struct TestModel {
    #[variable(causality = output, start = 1.0)]
    value: f64,
}

fn main() {
    println!("Generated ValueRef variants:");
    println!("Value = {:?}", ValueRef::Value);

    let model = TestModel::default();
    println!("Model: {:?}", model);
}
