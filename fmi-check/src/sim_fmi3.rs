use arrow::{
    array::{
        downcast_array, Float32Array, Float64Array, Int16Array, Int32Array, Int64Array, Int8Array,
        UInt16Array, UInt32Array, UInt64Array, UInt8Array,
    },
    datatypes::{DataType, Schema},
    record_batch::RecordBatch,
};
use fmi::{
    fmi3::instance::{
        traits::{CoSimulation, Common},
        DiscreteStates, Instance,
    },
    import::FmiImport,
};

use crate::input::FmiSchemaBuilder as _;

const FIXED_SOLVER_STEP: f64 = 1e-3;

pub struct InputState {
    input_schema: Schema,
    input_data: RecordBatch,
    continuous_inputs: Vec<(usize, u32)>,
    discrete_inputs: Vec<(usize, u32)>,
}

impl InputState {
    pub fn new(
        import: &fmi::fmi3::import::Fmi3,
        input_file: std::path::PathBuf,
    ) -> anyhow::Result<Self> {
        let md = import.model_description();
        let input_schema = md.model_variables.inputs_schema();

        let continuous_inputs = md.model_variables.continuous_inputs(&input_schema);
        let discrete_inputs = md.model_variables.discrete_inputs(&input_schema);
        let input_data = crate::input::csv_input(input_file, &input_schema)?;

        Ok(Self {
            input_schema,
            input_data,
            continuous_inputs,
            discrete_inputs,
        })
    }

    fn apply_inputs<Tag>(
        &self,
        time: f64,
        instance: &mut Instance<'_, Tag>,
        inputs: &[(usize, u32)],
    ) {
        let time_array: Float64Array =
            downcast_array(self.input_data.column_by_name("time").unwrap());
        let pre_lookup = crate::interpolation::pre_lookup(&time_array, time);

        for (column_index, value_reference) in inputs {
            let col = self.input_schema.field(*column_index);

            match col.data_type() {
                DataType::Boolean => todo!(),
                DataType::Int8 => {
                    let array: Int8Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_int8(&[*value_reference], &[value]);
                }
                DataType::Int16 => {
                    let array: Int16Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_int16(&[*value_reference], &[value]);
                }
                DataType::Int32 => {
                    let array: Int32Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_int32(&[*value_reference], &[value]);
                }
                DataType::Int64 => {
                    let array: Int64Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_int64(&[*value_reference], &[value]);
                }
                DataType::UInt8 => {
                    let array: UInt8Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_uint8(&[*value_reference], &[value]);
                }
                DataType::UInt16 => {
                    let array: UInt16Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_uint16(&[*value_reference], &[value]);
                }
                DataType::UInt32 => {
                    let array: UInt32Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_uint32(&[*value_reference], &[value]);
                }
                DataType::UInt64 => {
                    let array: UInt64Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_uint64(&[*value_reference], &[value]);
                }
                DataType::Float32 => {
                    let array: Float32Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_float32(&[*value_reference], &[value]);
                }
                DataType::Float64 => {
                    let array: Float64Array = downcast_array(self.input_data.column(*column_index));
                    let value = crate::interpolation::interpolate(&array, &pre_lookup);
                    instance.set_float64(&[*value_reference], &[value]);
                }
                DataType::Binary => todo!(),
                DataType::Utf8 => todo!(),
                _ => unimplemented!("Unsupported data type: {:?}", col.data_type()),
            }
        }
    }

    pub fn apply_continuous_inputs<Tag>(&self, time: f64, instance: &mut Instance<'_, Tag>) {
        self.apply_inputs(time, instance, &self.continuous_inputs);
    }

    pub fn apply_discrete_inputs<Tag>(&self, time: f64, instance: &mut Instance<'_, Tag>) {
        self.apply_inputs(time, instance, &self.discrete_inputs);
    }
}

pub fn co_simulation(
    import: &fmi::fmi3::import::Fmi3,
    start_time: f64,
    stop_time: f64,
    input_file: Option<std::path::PathBuf>,
) -> anyhow::Result<()> {
    let mut inst = import.instantiate_cs("inst1", true, true, true, true, &[])?;
    let input_state = input_file
        .map(|path| InputState::new(import, path))
        .transpose()?;

    // set start values
    //CALL(applyStartValues(S));

    let mut time = start_time;

    // initialize the FMU
    inst.enter_initialization_mode(None, time, Some(stop_time))
        .ok()?;

    // apply continuous and discrete inputs
    if let Some(input_state) = input_state {
        input_state.apply_continuous_inputs(time, &mut inst);
        input_state.apply_discrete_inputs(time, &mut inst);
    }

    inst.exit_initialization_mode().ok()?;

    let mut states = DiscreteStates::default();

    // update discrete states
    let terminate = loop {
        inst.update_discrete_states(&mut states).ok()?;

        if states.terminate_simulation {
            break false;
        }

        if !states.discrete_states_need_update {
            break true;
        }
    };

    inst.enter_step_mode().ok()?;

    // communication step size
    let step_size = 10.0 * FIXED_SOLVER_STEP;

    loop {
        //CALL(recordVariables(S, outputFile));

        if (states.terminate_simulation || time >= stop_time) {
            break;
        }

        let mut event_encountered = false;
        let mut early_return = false;

        inst.do_step(
            time,
            step_size,
            true,
            &mut event_encountered,
            &mut states.terminate_simulation,
            &mut early_return,
            &mut time,
        )
        .ok()?;

        if event_encountered {
            // record variables before event update
            //CALL(recordVariables(S, outputFile));

            // enter Event Mode
            inst.enter_event_mode().ok()?;

            // apply continuous and discrete inputs
            //CALL(applyContinuousInputs(S, true));
            //CALL(applyDiscreteInputs(S));

            // update discrete states
            loop {
                inst.update_discrete_states(&mut states).ok()?;

                if states.terminate_simulation {
                    break;
                }

                if !states.discrete_states_need_update {
                    break;
                }
            }

            // return to Step Mode
            inst.enter_step_mode().ok()?;
        }
    }

    Ok(())
}
