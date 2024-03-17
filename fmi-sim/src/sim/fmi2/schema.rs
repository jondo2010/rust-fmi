use arrow::{
    array::StringArray,
    datatypes::{Field, Fields, Schema},
};
use fmi::{
    fmi2::{
        import::Fmi2Import,
        schema::{Causality, Variability},
    },
    traits::FmiImport,
};

use crate::sim::{io::StartValues, traits::FmiSchemaBuilder};

impl FmiSchemaBuilder for Fmi2Import
where
    Self::ValueReference: From<u32>,
{
    fn inputs_schema(&self) -> Schema {
        let input_fields = self
            .model_description()
            .model_variables
            .variables
            .iter()
            .filter(|v| v.causality == Causality::Input)
            .map(|v| Field::new(&v.name, v.elem.data_type(), false))
            .collect::<Fields>();

        Schema::new(input_fields)
    }

    fn outputs_schema(&self) -> Schema {
        let time = Field::new("time", arrow::datatypes::DataType::Float64, false);
        let output_fields = self
            .model_description()
            .model_variables
            .variables
            .iter()
            .filter(|v| v.causality == Causality::Output)
            .map(|v| Field::new(&v.name, v.elem.data_type(), false))
            .chain(std::iter::once(time))
            .collect::<Fields>();

        Schema::new(output_fields)
    }

    fn continuous_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueReference)> + '_ {
        self.model_description()
            .model_variables
            .variables
            .iter()
            .filter(|v| v.causality == Causality::Input && v.variability == Variability::Continuous)
            .map(|v| {
                (
                    Field::new(&v.name, v.elem.data_type(), false),
                    v.value_reference,
                )
            })
    }

    fn discrete_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueReference)> + '_ {
        self.model_description()
            .model_variables
            .variables
            .iter()
            .filter(|v| {
                v.causality == Causality::Input
                    && (v.variability == Variability::Discrete
                        || v.variability == Variability::Tunable)
            })
            .map(|v| {
                (
                    Field::new(&v.name, v.elem.data_type(), false),
                    v.value_reference,
                )
            })
    }

    fn outputs(&self) -> impl Iterator<Item = (Field, Self::ValueReference)> + '_ {
        self.model_description()
            .model_variables
            .variables
            .iter()
            .filter(|v| v.causality == Causality::Output)
            .map(|v| {
                (
                    Field::new(&v.name, v.elem.data_type(), false),
                    v.value_reference,
                )
            })
    }

    fn parse_start_values(
        &self,
        start_values: &[String],
    ) -> anyhow::Result<crate::sim::io::StartValues<Self::ValueReference>> {
        let mut variables = vec![];

        for start_value in start_values {
            let (name, value) = start_value
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid start value: {}", start_value))?;

            let var = self
                .model_description()
                .model_variables
                .variables
                .iter()
                .find(|v| v.name == name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Invalid variable name: {name}. Valid variables are: {valid:?}",
                        valid = self
                            .model_description()
                            .model_variables
                            .variables
                            .iter()
                            .map(|v| &v.name)
                            .collect::<Vec<_>>()
                    )
                })?;

            let dt = var.elem.data_type();
            let ary = StringArray::from(vec![value.to_string()]);
            let ary = arrow::compute::cast(&ary, &dt)
                .map_err(|e| anyhow::anyhow!("Error casting type: {e}"))?;

            variables.push((var.value_reference, ary));
        }

        Ok(StartValues {
            structural_parameters: vec![],
            variables,
        })
    }
}
