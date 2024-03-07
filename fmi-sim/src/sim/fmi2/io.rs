use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use fmi::fmi2::instance::Common;

use crate::sim::{
    traits::{FmiSchemaBuilder, InstanceSetValues},
    util::project_input_data,
    InputState,
};
