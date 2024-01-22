use arrow::{
    array::{self, Float64Array, StringArray},
    datatypes::{DataType, Schema},
    downcast_primitive_array,
    record_batch::RecordBatch,
};
use fmi::{
    fmi3::{import::Fmi3Import, instance::Instance},
    FmiInstance,
};

use super::{
    interpolation::{self, Interpolate, Linear, PreLookup},
    schema_builder::FmiSchemaBuilder,
    InstanceSetValues,
};

pub struct InputState {
    input_schema: Schema,
    input_data: Option<RecordBatch>,
    continuous_inputs: Vec<(usize, u32)>,
    discrete_inputs: Vec<(usize, u32)>,
}

impl InputState {
    pub fn new(import: &Fmi3Import, input_data: Option<RecordBatch>) -> Self {
        let input_schema = import.inputs_schema();
        let continuous_inputs = import.continuous_inputs(&input_schema);
        let discrete_inputs = import.discrete_inputs(&input_schema);

        Self {
            input_schema,
            input_data,
            continuous_inputs,
            discrete_inputs,
        }
    }

    pub fn apply_input<Tag, I: Interpolate>(
        &self,
        time: f64,
        instance: &mut Instance<'_, Tag>,
        discrete: bool,
        continuous: bool,
        after_event: bool,
    ) -> anyhow::Result<()> {
        if let Some(input_data) = &self.input_data {
            let time_array: Float64Array =
                array::downcast_array(input_data.column_by_name("time").unwrap());

            if continuous {
                let pl = PreLookup::new(&time_array, time, after_event);

                for (col_idx, vr) in &self.continuous_inputs {
                    let field = self.input_schema.field(*col_idx);

                    if let Some(input_col) = input_data.column_by_name(field.name()) {
                        log::trace!(
                            "Applying continuous input {}={input_col:?} at time {time}",
                            field.name()
                        );

                        let ary = arrow::compute::cast(input_col, &field.data_type())
                            .map_err(|_| anyhow::anyhow!("Error casting type"))?;

                        instance.set_interpolated::<I>(*vr, &pl, &ary)?;
                    }
                }
            }

            if discrete {
                //TODO: Refactor the interpolation code to separate index lookup from interpolation
                let input_idx = interpolation::find_index(&time_array, time, after_event);

                for (col_idx, vr) in &self.discrete_inputs {
                    let field = self.input_schema.field(*col_idx);

                    if let Some(input_col) = input_data.column_by_name(field.name()) {
                        log::trace!(
                            "Applying discrete input {}={input_col:?} at time {time}",
                            field.name()
                        );

                        let ary = arrow::compute::cast(input_col, &field.data_type())
                            .map_err(|_| anyhow::anyhow!("Error casting type"))?;

                        instance.set_array(&[*vr], &ary.slice(input_idx, 1));
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse the start values from the command line and set them in the FMU.
    pub fn apply_start_values<Tag>(
        &self,
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

    pub fn next_input_event(&self, time: f64) -> f64 {
        if let Some(input_data) = &self.input_data {
            let time_array: Float64Array =
                array::downcast_array(input_data.column_by_name("time").unwrap());

            for i in 0..(time_array.len() - 1) {
                let t0 = time_array.value(i);
                let t1 = time_array.value(i + 1);

                if time >= t1 {
                    continue;
                }

                if t0 == t1 {
                    return t0; // discrete change of a continuous variable
                }

                //TODO: This could be computed once and cached

                // skip continuous variables
                for (col_idx, vr) in &self.discrete_inputs {
                    let field = self.input_schema.field(*col_idx);

                    if let Some(input_col) = input_data.column_by_name(field.name()) {
                        use arrow::datatypes as arrow_schema;
                        if downcast_primitive_array!(
                            input_col => input_col.value(i) != input_col.value(i + 1),
                            t => panic!("Unsupported datatype {}", t)
                        ) {
                            return t1;
                        }
                    }
                }
            }
        }

        return f64::INFINITY;
    }
}
