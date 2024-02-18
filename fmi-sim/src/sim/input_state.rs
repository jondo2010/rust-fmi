use std::sync::Arc;

use anyhow::Context;
use arrow::{
    array::{self, Float64Array, StringArray},
    datatypes::{Field, Schema, SchemaRef},
    downcast_primitive_array,
    record_batch::RecordBatch,
    util,
};
use comfy_table::Table;
use fmi::fmi3::{import::Fmi3Import, instance::Instance};
use itertools::Itertools;

use super::{
    interpolation::{self, Interpolate, PreLookup},
    schema_builder::FmiSchemaBuilder,
    InstanceSetValues,
};

/// Format the projected fields in a human-readable format
fn pretty_format_projection(
    input_data_schema: Arc<Schema>,
    model_input_schema: Arc<Schema>,
    time_field: Arc<Field>,
) -> impl std::fmt::Display {
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::ASCII_BORDERS_ONLY_CONDENSED);
    table.set_header(vec!["Name", "Input Type", "Model Type"]);
    let rows_iter = input_data_schema.fields().iter().map(|input_field| {
        let model_field_name = model_input_schema
            .fields()
            .iter()
            .chain(std::iter::once(&time_field))
            .find(|model_field| model_field.name() == input_field.name())
            .map(|model_field| model_field.data_type());
        vec![
            input_field.name().to_string(),
            input_field.data_type().to_string(),
            model_field_name
                .map(|t| t.to_string())
                .unwrap_or("-None-".to_string()),
        ]
    });
    table.add_rows(rows_iter);
    table
}

/// Transform the `input_data` to match the `model_input_schema`. Input data columns are projected and
/// cast to the corresponding input schema columns.
///
/// This is necessary because the `input_data` may have extra columns or have different datatypes.
fn project_input_data(
    input_data: &RecordBatch,
    model_input_schema: SchemaRef,
) -> anyhow::Result<RecordBatch> {
    let input_data_schema = input_data.schema();

    let time_field = Arc::new(Field::new(
        "time",
        arrow::datatypes::DataType::Float64,
        false,
    ));

    let (projected_fields, projected_columns): (Vec<_>, Vec<_>) = model_input_schema
        .fields()
        .iter()
        .chain(std::iter::once(&time_field))
        .filter_map(|field| {
            input_data.column_by_name(field.name()).map(|col| {
                arrow::compute::cast(col, &field.data_type())
                    .map(|col| (field.clone(), col))
                    .map_err(|_| anyhow::anyhow!("Error casting type"))
            })
        })
        .process_results(|pairs| pairs.unzip())?;

    log::debug!(
        "Projected input data schema:\n{}",
        pretty_format_projection(input_data_schema, model_input_schema, time_field)
    );

    let input_data_schema = Arc::new(Schema::new(projected_fields));
    RecordBatch::try_new(input_data_schema, projected_columns).map_err(anyhow::Error::from)
}

pub struct InputState {
    input_data: Option<RecordBatch>,
    input_schema: SchemaRef,
    // Map schema column index to ValueReference
    continuous_inputs: Vec<(usize, u32)>,
    // Map schema column index to ValueReference
    discrete_inputs: Vec<(usize, u32)>,
}

impl InputState {
    pub fn new(import: &Fmi3Import, input_data: Option<RecordBatch>) -> anyhow::Result<Self> {
        let model_input_schema = Arc::new(import.inputs_schema());
        let continuous_inputs = import.continuous_inputs(&model_input_schema);
        let discrete_inputs = import.discrete_inputs(&model_input_schema);

        let input_data = if let Some(input_data) = input_data {
            let rb = project_input_data(&input_data, model_input_schema.clone())?;

            log::debug!(
                "Input data:\n{}",
                util::pretty::pretty_format_batches(&[input_data])?
            );

            Some(rb)
        } else {
            None
        };

        Ok(Self {
            input_schema: model_input_schema,
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
            let time_array: Float64Array = array::downcast_array(
                input_data
                    .column_by_name("time")
                    .context("Input data must have a column named 'time' with the time values")?,
            );

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

    /// Parse a list of "var=value" strings into a list of (ValueReference, Array) tuples, suitable for `apply_start_values`.
    pub fn parse_start_values(
        &self,
        start_values: &[String],
    ) -> anyhow::Result<Vec<(u32, array::ArrayRef)>> {
        start_values
            .iter()
            .map(|start_value| {
                let (name, value) = start_value
                    .split_once('=')
                    .ok_or_else(|| anyhow::anyhow!("Invalid start value"))?;

                let index = self.input_schema.index_of(name).map_err(|_| {
                    anyhow::anyhow!(
                        "Invalid variable name: {name}. Valid variables are: {valid:?}",
                        valid = self
                            .input_schema
                            .fields()
                            .iter()
                            .map(|field| field.name().to_string())
                            .collect::<Vec<_>>()
                    )
                })?;

                let dt = self.input_schema.field(index).data_type();
                let ary = StringArray::from(vec![value.to_string()]);
                let ary = arrow::compute::cast(&ary, dt)
                    .map_err(|_| anyhow::anyhow!("Error casting type"))?;

                let vr = self
                    .continuous_inputs
                    .iter()
                    .chain(self.discrete_inputs.iter())
                    .find_map(|(col_idx, vr)| (col_idx == &index).then(|| *vr))
                    .ok_or_else(|| anyhow::anyhow!("VR not found"))?;

                Ok((vr, ary))
            })
            .collect::<anyhow::Result<Vec<_>>>()
    }

    /// Apply a list of (ValueReference, Array) tuples to the instance.
    pub fn apply_start_values<Tag>(
        &self,
        instance: &mut Instance<'_, Tag>,
        start_values: impl IntoIterator<Item = (u32, array::ArrayRef)>,
    ) -> anyhow::Result<()> {
        for (vr, ary) in start_values {
            log::trace!("Setting start value `{}`", vr);
            instance.set_array(&[vr], &ary);
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
