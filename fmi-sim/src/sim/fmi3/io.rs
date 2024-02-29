//! FMI3-specific input and output implementation

use std::sync::Arc;

use anyhow::Context;
use arrow::{
    array::{
        downcast_array, make_builder, BinaryBuilder, BooleanBuilder, Float32Builder, Float64Array,
        Float64Builder, Int16Builder, Int32Builder, Int64Builder, Int8Builder, UInt16Builder,
        UInt32Builder, UInt64Builder, UInt8Builder,
    },
    datatypes::DataType,
    downcast_primitive_array,
    record_batch::RecordBatch,
};
use fmi::fmi3::instance::Common;

use crate::sim::{
    interpolation::{find_index, Interpolate, PreLookup},
    io::Recorder,
    params::SimParams,
    traits::{FmiSchemaBuilder, InstanceSetValues, SimInput, SimOutput},
    util::project_input_data,
    InputState, OutputState,
};

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
                arrow::util::pretty::pretty_format_batches(&[input_data])?
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

    fn apply_input<I: Interpolate>(
        &mut self,
        time: f64,
        inst: &mut Self::Inst,
        discrete: bool,
        continuous: bool,
        after_event: bool,
    ) -> anyhow::Result<()> {
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
                let input_idx = find_index(&time_array, time, after_event);

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

macro_rules! impl_recorder {
    ($getter:ident, $builder_type:ident, $inst:expr, $vr:ident, $builder:ident) => {{
        let mut value = [std::default::Default::default()];
        $inst.$getter(&[*$vr], &mut value).ok()?;
        $builder
            .as_any_mut()
            .downcast_mut::<$builder_type>()
            .expect(concat!("column is not ", stringify!($builder_type)))
            .append_value(value[0]);
    }};
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
                let builder = make_builder(field.data_type(), num_points);
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
                    let mut value = [std::default::Default::default()];
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
