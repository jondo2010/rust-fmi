use std::default;
use std::sync::Arc;

use anyhow::Context;
use arrow::{
    array::{
        self, downcast_array, ArrayBuilder, ArrayRef, BinaryBuilder, BooleanBuilder,
        Float32Builder, Float64Array, Float64Builder, Int16Builder, Int32Builder, Int64Builder,
        Int8Builder, UInt16Builder, UInt32Builder, UInt64Builder, UInt8Builder,
    },
    datatypes::{DataType, Field, Schema, SchemaRef},
    downcast_primitive_array,
    record_batch::RecordBatch,
    util,
};
use comfy_table::Table;
use fmi::{fmi3::instance::Common, traits::FmiInstance};
use itertools::Itertools;

use super::{
    interpolation::{self, Interpolate, PreLookup},
    params::SimParams,
    traits::{FmiSchemaBuilder, InstanceSetValues, SimInput, SimOutput},
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
                arrow::compute::cast(col, field.data_type())
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

/// Container for holding initial values for the FMU.
pub struct StartValues<VR> {
    pub structural_parameters: Vec<(VR, ArrayRef)>,
    pub variables: Vec<(VR, ArrayRef)>,
}

pub struct InputState<Inst: FmiInstance> {
    pub(crate) input_data: Option<RecordBatch>,
    // Map schema column index to ValueReference
    pub(crate) continuous_inputs: Vec<(Field, Inst::ValueReference)>,
    // Map schema column index to ValueReference
    pub(crate) discrete_inputs: Vec<(Field, Inst::ValueReference)>,
}

macro_rules! impl_recorder {
    ($getter:ident, $builder_type:ident, $inst:expr, $vr:ident, $builder:ident) => {{
        let mut value = [default::Default::default()];
        $inst.$getter(&[*$vr], &mut value).ok()?;
        $builder
            .as_any_mut()
            .downcast_mut::<$builder_type>()
            .expect(concat!("column is not ", stringify!($builder_type)))
            .append_value(value[0]);
    }};
}

pub struct Recorder<Inst: FmiInstance> {
    field: Field,
    value_reference: Inst::ValueReference,
    builder: Box<dyn ArrayBuilder>,
}

pub struct OutputState<Inst: FmiInstance> {
    time: Float64Builder,
    recorders: Vec<Recorder<Inst>>,
}

impl<Inst> OutputState<Inst>
where
    Inst: Common,
    Inst::Import: FmiSchemaBuilder,
{
    /// Finish the output state and return the RecordBatch.
    pub fn finish(self) -> RecordBatch {
        let Self {
            mut time,
            recorders,
        } = self;

        let recorders = recorders.into_iter().map(
            |Recorder {
                 field,
                 value_reference: _,
                 mut builder,
             }| { (field, builder.finish()) },
        );

        let time = std::iter::once((
            Field::new("time", DataType::Float64, false),
            Arc::new(time.finish()) as _,
        ));

        let (fields, columns): (Vec<_>, Vec<_>) = time.chain(recorders).unzip();
        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(schema, columns).unwrap()
    }
}

impl<Inst> SimInput for InputState<Inst>
where
    Inst: Common + InstanceSetValues,
    Inst::Import: FmiSchemaBuilder,
{
    type Inst = Inst;

    fn new(import: &Inst::Import, input_data: Option<RecordBatch>) -> anyhow::Result<Self> {
        let model_input_schema = Arc::new(import.inputs_schema());
        let continuous_inputs = import.continuous_inputs().collect();
        let discrete_inputs = import.discrete_inputs().collect();

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
            input_data,
            continuous_inputs,
            discrete_inputs,
        })
    }

    fn apply_input<I>(
        &mut self,
        time: f64,
        inst: &mut Self::Inst,
        discrete: bool,
        continuous: bool,
        after_event: bool,
    ) -> anyhow::Result<()>
    where
        I: Interpolate,
    {
        if let Some(input_data) = &self.input_data {
            let time_array: Float64Array = downcast_array(
                input_data
                    .column_by_name("time")
                    .context("Input data must have a column named 'time' with the time values")?,
            );

            if continuous {
                let pl = PreLookup::new(&time_array, time, after_event);

                for (field, vr) in &self.continuous_inputs {
                    if let Some(input_col) = input_data.column_by_name(field.name()) {
                        log::trace!(
                            "Applying continuous input {}={input_col:?} at time {time}",
                            field.name()
                        );

                        let ary = arrow::compute::cast(input_col, field.data_type())
                            .map_err(|_| anyhow::anyhow!("Error casting type"))?;

                        inst.set_interpolated::<I>(*vr, &pl, &ary)?;
                    }
                }
            }

            if discrete {
                // TODO: Refactor the interpolation code to separate index lookup from interpolation
                let input_idx = interpolation::find_index(&time_array, time, after_event);

                for (field, vr) in &self.discrete_inputs {
                    if let Some(input_col) = input_data.column_by_name(field.name()) {
                        log::trace!(
                            "Applying discrete input {}={input_col:?} at time {time}",
                            field.name()
                        );

                        let ary = arrow::compute::cast(input_col, field.data_type())
                            .map_err(|_| anyhow::anyhow!("Error casting type"))?;

                        inst.set_array(&[*vr], &ary.slice(input_idx, 1));
                    }
                }
            }
        }

        Ok(())
    }

    fn next_input_event(&self, time: f64) -> f64 {
        if let Some(input_data) = &self.input_data {
            let time_array: Float64Array =
                downcast_array(input_data.column_by_name("time").unwrap());

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
                for (field, _vr) in &self.discrete_inputs {
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
        f64::INFINITY
    }
}

impl<Inst> SimOutput for OutputState<Inst>
where
    Inst: Common,
    Inst::Import: FmiSchemaBuilder,
{
    type Inst = Inst;

    fn new(import: &Inst::Import, sim_params: &SimParams) -> Self {
        let num_points = ((sim_params.stop_time - sim_params.start_time)
            / sim_params.output_interval)
            .ceil() as usize;

        let time = Float64Builder::with_capacity(num_points);

        let recorders = import
            .outputs()
            .map(|(field, vr)| {
                let builder = array::make_builder(field.data_type(), num_points);
                Recorder {
                    field,
                    value_reference: vr,
                    builder,
                }
            })
            .collect();

        Self { time, recorders }
    }

    fn record_outputs(&mut self, time: f64, inst: &mut Self::Inst) -> anyhow::Result<()> {
        log::trace!("Recording variables at time {}", time);

        self.time.append_value(time);

        for Recorder {
            field,
            value_reference: vr,
            builder,
        } in &mut self.recorders
        {
            match field.data_type() {
                DataType::Boolean => {
                    impl_recorder!(get_boolean, BooleanBuilder, inst, vr, builder)
                }
                DataType::Int8 => {
                    impl_recorder!(get_int8, Int8Builder, inst, vr, builder)
                }
                DataType::Int16 => {
                    impl_recorder!(get_int16, Int16Builder, inst, vr, builder)
                }
                DataType::Int32 => {
                    impl_recorder!(get_int32, Int32Builder, inst, vr, builder)
                }
                DataType::Int64 => {
                    impl_recorder!(get_int64, Int64Builder, inst, vr, builder)
                }
                DataType::UInt8 => {
                    impl_recorder!(get_uint8, UInt8Builder, inst, vr, builder)
                }
                DataType::UInt16 => {
                    impl_recorder!(get_uint16, UInt16Builder, inst, vr, builder)
                }
                DataType::UInt32 => {
                    impl_recorder!(get_uint32, UInt32Builder, inst, vr, builder)
                }
                DataType::UInt64 => {
                    impl_recorder!(get_uint64, UInt64Builder, inst, vr, builder)
                }
                DataType::Float32 => {
                    impl_recorder!(get_float32, Float32Builder, inst, vr, builder)
                }
                DataType::Float64 => {
                    impl_recorder!(get_float64, Float64Builder, inst, vr, builder)
                }
                DataType::Binary => {
                    let mut value = [default::Default::default()];
                    inst.get_binary(&[*vr], &mut value).ok()?;
                    let [value] = value;
                    builder
                        .as_any_mut()
                        .downcast_mut::<BinaryBuilder>()
                        .expect("column is not Binary")
                        .append_value(value);
                }
                _ => unimplemented!("Unsupported data type: {:?}", field.data_type()),
            }
        }

        Ok(())
    }
}
