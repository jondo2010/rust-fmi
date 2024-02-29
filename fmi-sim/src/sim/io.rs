use std::sync::Arc;

use arrow::{
    array::{ArrayBuilder, ArrayRef, Float64Builder},
    datatypes::{DataType, Field, Schema},
    record_batch::RecordBatch,
};
use fmi::traits::FmiInstance;

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

pub struct Recorder<Inst: FmiInstance> {
    pub(crate) field: Field,
    pub(crate) value_reference: Inst::ValueReference,
    pub(crate) builder: Box<dyn ArrayBuilder>,
}

pub struct OutputState<Inst: FmiInstance> {
    pub(crate) time: Float64Builder,
    pub(crate) recorders: Vec<Recorder<Inst>>,
}

impl<Inst> OutputState<Inst>
where
    Inst: FmiInstance,
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
