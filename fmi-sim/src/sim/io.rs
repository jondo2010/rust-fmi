use std::{fmt::Display, sync::Arc};

use anyhow::Context;
use arrow::{
    array::{ArrayBuilder, ArrayRef, Float64Array, Float64Builder, downcast_array, make_builder},
    datatypes::{DataType, Field, Schema},
    downcast_primitive_array,
    record_batch::RecordBatch,
};
use fmi::traits::FmiInstance;

use crate::Error;

use super::{
    interpolation::{Interpolate, PreLookup, find_index},
    params::SimParams,
    traits::{ImportSchemaBuilder, InstSetValues},
    util::project_input_data,
};

/// Container for holding initial values for the FMU.
pub struct StartValues<VR> {
    pub structural_parameters: Vec<(VR, ArrayRef)>,
    pub variables: Vec<(VR, ArrayRef)>,
}

pub struct InputState<Inst: FmiInstance> {
    pub(crate) input_data: Option<RecordBatch>,
    // Map schema column index to ValueReference
    pub(crate) continuous_inputs: Vec<(Field, Inst::ValueRef)>,
    // Map schema column index to ValueReference
    pub(crate) discrete_inputs: Vec<(Field, Inst::ValueRef)>,
}

impl<Inst> Display for InputState<Inst>
where
    Inst: FmiInstance,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let continuous_inputs = self
            .continuous_inputs
            .iter()
            .map(|(field, _)| field.name())
            .collect::<Vec<_>>();

        let discrete_inputs = self
            .discrete_inputs
            .iter()
            .map(|(field, _)| field.name())
            .collect::<Vec<_>>();

        f.write_str("InputState {\n")?;
        if let Some(input_data) = &self.input_data {
            writeln!(
                f,
                "input_data:\n{}",
                arrow::util::pretty::pretty_format_batches(std::slice::from_ref(input_data))
                    .unwrap()
            )?;
        } else {
            writeln!(f, "input_data: None")?;
        }
        writeln!(f, "continuous_inputs: {continuous_inputs:?}")?;
        writeln!(f, "discrete_inputs: {discrete_inputs:?}")?;
        write!(f, "}}")
    }
}

impl<Inst> InputState<Inst>
where
    Inst: FmiInstance,
{
    pub fn new<Import: ImportSchemaBuilder<ValueRef = Inst::ValueRef>>(
        import: &Import,
        input_data: Option<RecordBatch>,
    ) -> anyhow::Result<Self> {
        let model_input_schema = Arc::new(import.inputs_schema());
        let continuous_inputs = import.continuous_inputs().collect();
        let discrete_inputs = import.discrete_inputs().collect();

        let input_data = input_data
            .map(|input_data| project_input_data(&input_data, model_input_schema.clone()))
            .transpose()?;

        Ok(Self {
            input_data,
            continuous_inputs,
            discrete_inputs,
        })
    }
}

impl<Inst> InputState<Inst>
where
    Inst: InstSetValues,
{
    pub fn apply_input<I: Interpolate>(
        &mut self,
        time: f64,
        inst: &mut Inst,
        discrete: bool,
        continuous: bool,
        after_event: bool,
    ) -> Result<(), Error> {
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
                        assert_eq!(input_col.data_type(), field.data_type());

                        //let ary = arrow::compute::cast(input_col, field.data_type()).map_err(|_| anyhow::anyhow!("Error casting type"))?;

                        //log::trace!( "Applying continuous input {}={input_col:?} at time {time}", field.name());

                        inst.set_interpolated::<I>(*vr, &pl, input_col)?;
                    }
                }
            }

            if discrete {
                // TODO: Refactor the interpolation code to separate index lookup from interpolation
                let input_idx = find_index(&time_array, time, after_event);

                for (field, vr) in &self.discrete_inputs {
                    if let Some(input_col) = input_data.column_by_name(field.name()) {
                        let ary = arrow::compute::cast(input_col, field.data_type())
                            .map_err(|_| anyhow::anyhow!("Error casting type"))?;

                        let values = &ary.slice(input_idx, 1);

                        //log::trace!( "Applying discrete input {}={values:#?} at time {time:.2}", field.name());

                        inst.set_array(&[*vr], values);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn next_input_event(&self, time: f64) -> f64 {
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

pub struct Recorder<Inst: FmiInstance> {
    pub(crate) field: Field,
    pub(crate) value_reference: Inst::ValueRef,
    pub(crate) builder: Box<dyn ArrayBuilder>,
}

pub struct RecorderState<Inst: FmiInstance> {
    pub(crate) time: Float64Builder,
    pub(crate) recorders: Vec<Recorder<Inst>>,
}

impl<Inst> RecorderState<Inst>
where
    Inst: FmiInstance,
{
    pub fn new<Import: ImportSchemaBuilder<ValueRef = Inst::ValueRef>>(
        import: &Import,
        sim_params: &SimParams,
    ) -> Self {
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
