use std::{io::Seek as _, path::Path, sync::Arc};

use arrow::{
    array::{
        self, ArrayBuilder, Float32Array, Float32Builder, Float64Array, Float64Builder, Int16Array,
        Int16Builder, Int32Array, Int32Builder, Int64Array, Int64Builder, Int8Array, Int8Builder,
        UInt16Array, UInt16Builder, UInt32Array, UInt32Builder, UInt64Array, UInt64Builder,
        UInt8Array, UInt8Builder,
    },
    csv::{reader::Format, ReaderBuilder},
    datatypes::{DataType, Field, Fields, Schema},
    record_batch::RecordBatch,
};
use fmi::{
    fmi3::{
        import::Fmi3Import,
        instance::{Common as _, Instance},
    },
    FmiImport as _,
};

use crate::interpolation::{Interpolate, Linear};

pub trait FmiSchemaBuilder {
    /// Build the schema for the inputs of the model.
    fn inputs_schema(&self) -> Schema;
    /// Build the schema for the outputs of the model.
    fn outputs_schema(&self) -> Schema;
    /// Build a list of Schema column indices and value references for the continuous inputs.
    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
    /// Build a list of Schema column indices and value references for the discrete inputs.
    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
    /// Build a list of Schema column indices and value references for the outputs.
    fn outputs(&self, schema: &Schema) -> Vec<(usize, u32)>;
}

impl FmiSchemaBuilder for Fmi3Import {
    fn inputs_schema(&self) -> Schema {
        let input_fields = self
            .model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == fmi::fmi3::schema::Causality::Input)
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
            .filter(|v| v.causality() == fmi::fmi3::schema::Causality::Output)
            .map(|v| Field::new(v.name(), v.data_type().into(), false))
            .chain(std::iter::once(time))
            .collect::<Fields>();

        Schema::new(output_fields)
    }

    fn continuous_inputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter_map(|v| {
                (v.causality() == fmi::fmi3::schema::Causality::Input
                    && v.variability() == fmi::fmi3::schema::Variability::Continuous)
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }

    fn discrete_inputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter_map(|v| {
                (v.causality() == fmi::fmi3::schema::Causality::Input
                    && v.variability() == fmi::fmi3::schema::Variability::Discrete)
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }

    fn outputs(&self, schema: &Schema) -> Vec<(usize, u32)> {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter_map(|v| {
                (v.causality() == fmi::fmi3::schema::Causality::Output)
                    .then(|| (schema.index_of(v.name()).unwrap(), v.value_reference()))
            })
            .collect::<Vec<_>>()
    }
}

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
        let pl = crate::interpolation::PreLookup::new(&time_array, time);

        for (column_index, value_reference) in inputs {
            let col = self.input_schema.field(*column_index);

            match col.data_type() {
                DataType::Boolean => todo!(),
                DataType::Int8 => {
                    let array: Int8Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_int8(&[*value_reference], &[value]).ok()?;
                }
                DataType::Int16 => {
                    let array: Int16Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_int16(&[*value_reference], &[value]).ok()?;
                }
                DataType::Int32 => {
                    let array: Int32Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_int32(&[*value_reference], &[value]).ok()?;
                }
                DataType::Int64 => {
                    let array: Int64Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_int64(&[*value_reference], &[value]).ok()?;
                }
                DataType::UInt8 => {
                    let array: UInt8Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_uint8(&[*value_reference], &[value]).ok()?;
                }
                DataType::UInt16 => {
                    let array: UInt16Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_uint16(&[*value_reference], &[value]).ok()?;
                }
                DataType::UInt32 => {
                    let array: UInt32Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_uint32(&[*value_reference], &[value]).ok()?;
                }
                DataType::UInt64 => {
                    let array: UInt64Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_uint64(&[*value_reference], &[value]).ok()?;
                }
                DataType::Float32 => {
                    let array: Float32Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_float32(&[*value_reference], &[value]).ok()?;
                }
                DataType::Float64 => {
                    let array: Float64Array =
                        array::downcast_array(self.input_data.column(*column_index));
                    let value = I::interpolate(&pl, &array);
                    instance.set_float64(&[*value_reference], &[value]).ok()?;
                }
                DataType::Binary => todo!(),
                DataType::Utf8 => todo!(),
                _ => unimplemented!("Unsupported data type: {:?}", col.data_type()),
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
}

pub struct OutputState {
    output_schema: Schema,
    data_builders: Vec<Box<dyn ArrayBuilder>>,
    outputs: Vec<(usize, u32)>,
}

impl OutputState {
    pub fn new(import: &Fmi3Import, num_points: usize) -> anyhow::Result<Self> {
        let output_schema = import.outputs_schema();

        let data_builders = output_schema
            .fields()
            .iter()
            .map(|field| array::make_builder(field.data_type(), num_points))
            .collect();

        let outputs = import.outputs(&output_schema);

        Ok(Self {
            output_schema,
            data_builders,
            outputs,
        })
    }

    pub fn record_variables(
        &mut self,
        inst: &mut Instance<'_, fmi::fmi3::instance::CS>,
        time: f64,
    ) -> anyhow::Result<()> {
        log::trace!("Recording variables at time {}", time);

        let time_idx = self
            .output_schema
            .index_of("time")
            .expect("time column not found");
        self.data_builders[time_idx]
            .as_any_mut()
            .downcast_mut::<Float64Builder>()
            .expect("time column is not Float64")
            .append_value(time);

        for (column_index, value_reference) in &self.outputs {
            let col = self.output_schema.field(*column_index);

            match col.data_type() {
                DataType::Boolean => todo!(),
                DataType::Int8 => {
                    let mut value = [0];
                    inst.get_int8(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int8Builder>()
                        .expect("column is not Int8")
                        .append_value(value[0]);
                }
                DataType::Int16 => {
                    let mut value = [0];
                    inst.get_int16(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int16Builder>()
                        .expect("column is not Int16")
                        .append_value(value[0]);
                }
                DataType::Int32 => {
                    let mut value = [0];
                    inst.get_int32(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int32Builder>()
                        .expect("column is not Int32")
                        .append_value(value[0]);
                }
                DataType::Int64 => {
                    let mut value = [0];
                    inst.get_int64(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Int64Builder>()
                        .expect("column is not Int64")
                        .append_value(value[0]);
                }
                DataType::UInt8 => {
                    let mut value = [0];
                    inst.get_uint8(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt8Builder>()
                        .expect("column is not UInt8")
                        .append_value(value[0]);
                }
                DataType::UInt16 => {
                    let mut value = [0];
                    inst.get_uint16(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt16Builder>()
                        .expect("column is not UInt16")
                        .append_value(value[0]);
                }
                DataType::UInt32 => {
                    let mut value = [0];
                    inst.get_uint32(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt32Builder>()
                        .expect("column is not UInt32")
                        .append_value(value[0]);
                }
                DataType::UInt64 => {
                    let mut value = [0];
                    inst.get_uint64(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<UInt64Builder>()
                        .expect("column is not UInt64")
                        .append_value(value[0]);
                }
                DataType::Float32 => {
                    let mut value = [0.0];
                    inst.get_float32(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Float32Builder>()
                        .expect("column is not Float32")
                        .append_value(value[0]);
                }
                DataType::Float64 => {
                    let mut value = [0.0];
                    inst.get_float64(&[*value_reference], &mut value).ok()?;
                    self.data_builders[*column_index]
                        .as_any_mut()
                        .downcast_mut::<Float64Builder>()
                        .expect("column is not Float64")
                        .append_value(value[0]);
                }
                _ => unimplemented!("Unsupported data type: {:?}", col.data_type()),
            }
        }

        Ok(())
    }

    /// Finish the output state and return the RecordBatch.
    pub fn finish(self) -> RecordBatch {
        let columns = self
            .data_builders
            .into_iter()
            .map(|mut builder| builder.finish())
            .collect::<Vec<_>>();

        RecordBatch::try_new(Arc::new(self.output_schema), columns)
            .expect("Failed to create RecordBatch")
    }
}

/// Read a CSV file into a single RecordBatch.
fn csv_recordbatch<P>(path: P, input_schema: &Schema) -> anyhow::Result<RecordBatch>
where
    P: AsRef<Path>,
{
    let mut file = std::fs::File::open(path)?;

    // Infer the schema with the first 100 records
    let (file_schema, _) = Format::default()
        .with_header(true)
        .infer_schema(&file, Some(100))?;
    file.rewind()?;

    let time = Arc::new(arrow::datatypes::Field::new(
        "time",
        arrow::datatypes::DataType::Float64,
        false,
    ));

    // Build a projection based on the input schema and the file schema.
    // Input fields that are not in the file schema are ignored.
    let input_projection = input_schema
        .fields()
        .iter()
        .chain(std::iter::once(&time))
        .filter_map(|input_field| {
            file_schema.index_of(input_field.name()).ok().map(|idx| {
                let file_dt = file_schema.field(idx).data_type();
                let input_dt = input_field.data_type();

                // Check if the file data type is compatible with the input data type.
                let dt_match = file_dt == input_dt
                    || file_dt.primitive_width() >= input_dt.primitive_width()
                        //&& file_dt.is_signed_integer() == input_dt.is_signed_integer()
                        //&& file_dt.is_unsigned_integer() == input_dt.is_unsigned_integer()
                        && file_dt.is_floating() == input_dt.is_floating();

                dt_match.then(|| idx).ok_or(anyhow::anyhow!(
                    "Input field {} has type {:?} but file field {} has type {:?}",
                    input_field.name(),
                    input_field.data_type(),
                    file_schema.field(idx).name(),
                    file_schema.field(idx).data_type()
                ))
            })
        })
        .collect::<Result<Vec<usize>, _>>()?;

    let reader = ReaderBuilder::new(Arc::new(file_schema))
        .with_header(true)
        .with_projection(input_projection)
        .build(file)?;

    let batches = reader.collect::<Result<Vec<_>, _>>()?;

    Ok(arrow::compute::concat_batches(
        &batches[0].schema(),
        &batches,
    )?)
}

#[test]
fn test_input_csv() {
    let import = fmi::Import::new("../data/reference_fmus/3.0/Feedthrough.fmu")
        .unwrap()
        .as_fmi3()
        .unwrap();

    let schema = import.inputs_schema();

    let data = csv_recordbatch("../data/feedthrough_in.csv", &schema).unwrap();

    println!(
        "{}",
        arrow::util::pretty::pretty_format_batches(&[data]).unwrap()
    );

    // let time_array: Float64Array =
    // arrow::array::downcast_array(data[0].column_by_name("time").unwrap());
}
