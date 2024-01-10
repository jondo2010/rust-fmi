use std::path::Path;

use arrow::{
    array::{self, Float64Array, StringArray},
    datatypes::Schema,
    record_batch::RecordBatch,
};
use fmi::{
    fmi3::{import::Fmi3Import, instance::Instance},
    FmiInstance,
};

use crate::io::InstanceSetValues;

use super::{
    interpolation::{Interpolate, Linear, PreLookup},
    schema_builder::FmiSchemaBuilder,
};

pub struct InputState {
    input_schema: Schema,
    input_data: RecordBatch,
    continuous_inputs: Vec<(usize, u32)>,
    discrete_inputs: Vec<(usize, u32)>,
}

impl InputState {
    pub fn new<P: AsRef<Path>>(import: &Fmi3Import, input_file: P) -> anyhow::Result<Self> {
        let input_schema = import.inputs_schema();
        let continuous_inputs = import.continuous_inputs(&input_schema);
        let discrete_inputs = import.discrete_inputs(&input_schema);
        let input_data = crate::io::csv_recordbatch(input_file, &input_schema)?;

        Ok(Self {
            input_schema,
            input_data,
            continuous_inputs,
            discrete_inputs,
        })
    }

    fn apply_inputs<Tag, I: Interpolate>(
        &self,
        time: f64,
        instance: &mut Instance<'_, Tag>,
        inputs: &[(usize, u32)],
    ) -> anyhow::Result<()> {
        let time_array: Float64Array =
            array::downcast_array(self.input_data.column_by_name("time").unwrap());
        let pl = PreLookup::new(&time_array, time);

        for (column_index, value_reference) in inputs {
            let col = self.input_schema.field(*column_index);
            if let Some(input_col) = self.input_data.column_by_name(col.name()) {
                log::trace!("Applying input {}={input_col:?} at time {time}", col.name());

                let ary = arrow::compute::cast(input_col, &col.data_type())
                    .map_err(|_| anyhow::anyhow!("Error casting type"))?;

                instance.set_interpolated::<I>(*value_reference, &pl, &ary)?;
            }
        }
        Ok(())
    }

    pub fn apply_continuous_inputs<Tag>(
        &self,
        time: f64,
        instance: &mut Instance<'_, Tag>,
    ) -> anyhow::Result<()> {
        self.apply_inputs::<_, Linear>(time, instance, &self.continuous_inputs)
    }

    pub fn apply_discrete_inputs<Tag>(
        &self,
        time: f64,
        instance: &mut Instance<'_, Tag>,
    ) -> anyhow::Result<()> {
        self.apply_inputs::<_, Linear>(time, instance, &self.discrete_inputs)
    }

    /// Parse the start values from the command line and set them in the FMU.
    pub fn apply_start_values<Tag>(
        instance: &mut Instance<'_, Tag>,
        start_values: &[String],
    ) -> anyhow::Result<()> {
        for start_value in start_values.into_iter() {
            let (name, value) = start_value
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid start value"))?;

            let var = instance
                .model_description()
                .model_variables
                .iter_abstract()
                .find(|var| var.name() == name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Invalid variable name: {name}. Valid variables are: {valid}",
                        valid = instance
                            .model_description()
                            .model_variables
                            .iter_abstract()
                            .map(|var| var.name())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })?;

            let ary = StringArray::from(vec![value.to_string()]);
            let ary = arrow::compute::cast(&ary, &var.data_type().into())
                .map_err(|_| anyhow::anyhow!("Error casting type"))?;

            log::trace!("Setting start value `{name}` = `{value}`");
            instance.set_array(&[var.value_reference()], &ary);
        }

        Ok(())
    }
}
