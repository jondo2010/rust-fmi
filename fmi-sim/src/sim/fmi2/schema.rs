use arrow::datatypes::{Field, Fields, Schema};
use fmi::{
    fmi2::{
        import::Fmi2Import,
        schema::{Causality, Variability},
    },
    traits::FmiImport,
};

use crate::sim::traits::FmiSchemaBuilder;

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
                    v.value_reference.into(),
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
                    v.value_reference.into(),
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
                    v.value_reference.into(),
                )
            })
    }

    fn parse_start_values(
        &self,
        start_values: &[String],
    ) -> anyhow::Result<crate::sim::io::StartValues<Self::ValueReference>> {
        todo!()
    }
}
