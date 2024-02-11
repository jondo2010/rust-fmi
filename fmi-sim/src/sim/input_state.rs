use std::sync::Arc;

use arrow::{
    array::{self, Float64Array, StringArray},
    datatypes::{Schema, SchemaRef},
    downcast_primitive_array,
    record_batch::RecordBatch,
};
use fmi::{
    fmi3::{import::Fmi3Import, instance::Instance},
    FmiInstance,
};
use itertools::Itertools;

use super::{
    interpolation::{self, Interpolate, PreLookup},
    schema_builder::FmiSchemaBuilder,
    InstanceSetValues,
};

pub struct InputState {
    input_schema: SchemaRef,
    input_data: Option<RecordBatch>,
    continuous_inputs: Vec<(usize, u32)>,
    discrete_inputs: Vec<(usize, u32)>,
}

/// Transform the `input_data` to match the `input_schema`. Input data columns are projected and
/// cast to the corresponding input schema columns.
///
/// This is necessary because the input_data may have extra columns or have different datatypes.
fn project_input_data(
    input_data: &RecordBatch,
    input_schema: SchemaRef,
) -> anyhow::Result<RecordBatch> {
    let (projected_fields, projected_columns): (Vec<_>, Vec<_>) = input_schema
        .fields()
        .iter()
        .filter_map(|field| {
            input_data.column_by_name(field.name()).map(|col| {
                arrow::compute::cast(col, &field.data_type())
                    .map(|col| (field.clone(), col))
                    .map_err(|_| anyhow::anyhow!("Error casting type"))
            })
        })
        .process_results(|pairs| pairs.unzip())?;

    // if there are extra columns in the input data, ignore them but issue a warning:
    if projected_fields.len() < input_data.num_columns() {
        let extra_columns = input_data
            .schema()
            .fields()
            .iter()
            .filter_map(|field| (!projected_fields.contains(field)).then(|| field.name().as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        log::warn!("Ignoring extra columns in input data: [{extra_columns}]");
    }

    let input_data_schema = Arc::new(Schema::new(projected_fields));

    log::info!("Projected input data schema: {input_schema:#?} -> {input_data_schema:#?}");
    RecordBatch::try_new(input_data_schema, projected_columns).map_err(anyhow::Error::from)
}

impl InputState {
    pub fn new(import: &Fmi3Import, input_data: Option<RecordBatch>) -> anyhow::Result<Self> {
        let input_schema = Arc::new(import.inputs_schema());
        let continuous_inputs = import.continuous_inputs(&input_schema);
        let discrete_inputs = import.discrete_inputs(&input_schema);

        let input_data = if let Some(input_data) = input_data {
            let rb = project_input_data(&input_data, input_schema.clone())?;

            arrow::util::pretty::print_batches(&[input_data]).unwrap();
            Some(rb)
        } else {
            None
        };

        Ok(Self {
            input_schema,
            input_data,
            continuous_inputs,
            discrete_inputs,
        })
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
                // TODO: Refactor the interpolation code to separate index lookup from interpolation
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

                // TODO: This could be computed once and cached

                // skip continuous variables
                for (col_idx, _vr) in &self.discrete_inputs {
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
