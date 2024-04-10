use arrow::{
    array::{ArrayRef, StringArray},
    datatypes::{DataType, Field, Fields, Schema},
};
use fmi::{
    fmi3::{import::Fmi3Import, schema::Causality},
    schema::fmi3::Variability,
    traits::FmiImport,
};

use crate::sim::{io::StartValues, traits::ImportSchemaBuilder};

impl ImportSchemaBuilder for Fmi3Import
where
    Self::ValueRef: From<u32>,
{
    fn inputs_schema(&self) -> Schema {
        let input_fields = self
            .model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == Causality::Input)
            .map(|v| Field::new(v.name(), v.data_type().into(), false))
            .collect::<Fields>();

        Schema::new(input_fields)
    }

    fn outputs_schema(&self) -> Schema {
        let time = Field::new("time", DataType::Float64, false);
        let output_fields = self
            .model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == Causality::Output)
            .map(|v| Field::new(v.name(), v.data_type().into(), false))
            .chain(std::iter::once(time))
            .collect::<Fields>();

        Schema::new(output_fields)
    }

    fn continuous_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> + '_ {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| {
                v.causality() == Causality::Input
                    && v.variability() == fmi::fmi3::schema::Variability::Continuous
            })
            .map(|v| {
                (
                    Field::new(v.name(), v.data_type().into(), false),
                    v.value_reference(),
                )
            })
    }

    fn discrete_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> + '_ {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| {
                v.causality() == Causality::Input
                    && (v.variability() == Variability::Discrete
                        || v.variability() == Variability::Tunable)
            })
            .map(|v| {
                (
                    Field::new(v.name(), v.data_type().into(), false),
                    v.value_reference(),
                )
            })
    }

    fn outputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == Causality::Output)
            .map(|v| {
                (
                    Field::new(v.name(), v.data_type().into(), false),
                    v.value_reference(),
                )
            })
    }

    fn parse_start_values(
        &self,
        start_values: &[String],
    ) -> anyhow::Result<StartValues<Self::ValueRef>> {
        let mut structural_parameters: Vec<(Self::ValueRef, ArrayRef)> = vec![];
        let mut variables: Vec<(Self::ValueRef, ArrayRef)> = vec![];

        for start_value in start_values {
            let (name, value) = start_value
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid start value"))?;

            let var = self
                .model_description()
                .model_variables
                .iter_abstract()
                .find(|v| v.name() == name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Invalid variable name: {name}. Valid variables are: {valid:?}",
                        valid = self
                            .model_description()
                            .model_variables
                            .iter_abstract()
                            .map(|v| v.name())
                            .collect::<Vec<_>>()
                    )
                })?;

            let dt = arrow::datatypes::DataType::from(var.data_type());
            let ary = StringArray::from(vec![value.to_string()]);
            let ary = arrow::compute::cast(&ary, &dt)
                .map_err(|e| anyhow::anyhow!("Error casting type: {e}"))?;

            if var.causality() == Causality::StructuralParameter {
                structural_parameters.push((var.value_reference(), ary));
            } else {
                variables.push((var.value_reference(), ary));
            }
        }

        Ok(StartValues {
            structural_parameters,
            variables,
        })
    }
}
