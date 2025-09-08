use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::fmi3::TypedArrayableVariableTrait;

use super::{
    AbstractVariableTrait, FmiBinary, FmiBoolean, FmiFloat32, FmiFloat64, FmiInt8, FmiInt16,
    FmiInt32, FmiInt64, FmiString, FmiUInt8, FmiUInt16, FmiUInt32, FmiUInt64,
};

#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
pub struct ModelVariables {
    #[yaserde(rename = "Float32")]
    pub float32: Vec<FmiFloat32>,
    #[yaserde(rename = "Float64")]
    pub float64: Vec<FmiFloat64>,
    #[yaserde(rename = "Int8")]
    pub int8: Vec<FmiInt8>,
    #[yaserde(rename = "UInt8")]
    pub uint8: Vec<FmiUInt8>,
    #[yaserde(rename = "Int16")]
    pub int16: Vec<FmiInt16>,
    #[yaserde(rename = "UInt16")]
    pub uint16: Vec<FmiUInt16>,
    #[yaserde(rename = "Int32")]
    pub int32: Vec<FmiInt32>,
    #[yaserde(rename = "UInt32")]
    pub uint32: Vec<FmiUInt32>,
    #[yaserde(rename = "Int64")]
    pub int64: Vec<FmiInt64>,
    #[yaserde(rename = "UInt64")]
    pub uint64: Vec<FmiUInt64>,
    #[yaserde(rename = "Boolean")]
    pub boolean: Vec<FmiBoolean>,
    #[yaserde(rename = "String")]
    pub string: Vec<FmiString>,
    #[yaserde(rename = "Binary")]
    pub binary: Vec<FmiBinary>,
}

impl ModelVariables {
    /// Returns the total number of variables in the model description
    pub fn len(&self) -> usize {
        self.iter_abstract().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over all the AbstractVariables in the model description
    pub fn iter_abstract(&self) -> impl Iterator<Item = &dyn AbstractVariableTrait> {
        itertools::chain!(
            self.float32.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.float64.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.int8.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint8.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.int16.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint16.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.int32.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint32.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.int64.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint64.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.boolean.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.string.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.binary.iter().map(|v| v as &dyn AbstractVariableTrait),
        )
    }

    /// Returns an iterator over all the float32 and float64 variables in the model description
    pub fn iter_floating(&self) -> impl Iterator<Item = &dyn TypedArrayableVariableTrait> {
        itertools::chain!(
            self.float32
                .iter()
                .map(|v| v as &dyn TypedArrayableVariableTrait),
            self.float64
                .iter()
                .map(|v| v as &dyn TypedArrayableVariableTrait),
        )
    }

    /// Finds a variable by its name.
    pub fn find_by_name(&self, name: &str) -> Option<&dyn AbstractVariableTrait> {
        self.iter_abstract().find(|v| v.name() == name)
    }
}

/// Append a variable to the given `ModelVariables` struct
pub trait AppendToModelVariables: AbstractVariableTrait {
    fn append_to_variables(self, variables: &mut ModelVariables);
}

impl AppendToModelVariables for FmiFloat32 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.float32.push(self);
    }
}

impl AppendToModelVariables for FmiFloat64 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.float64.push(self);
    }
}

impl AppendToModelVariables for FmiInt8 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.int8.push(self);
    }
}

impl AppendToModelVariables for FmiInt16 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.int16.push(self);
    }
}

impl AppendToModelVariables for FmiInt32 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.int32.push(self);
    }
}

impl AppendToModelVariables for FmiInt64 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.int64.push(self);
    }
}

impl AppendToModelVariables for FmiUInt8 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.uint8.push(self);
    }
}

impl AppendToModelVariables for FmiUInt16 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.uint16.push(self);
    }
}

impl AppendToModelVariables for FmiUInt32 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.uint32.push(self);
    }
}

impl AppendToModelVariables for FmiUInt64 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.uint64.push(self);
    }
}

impl AppendToModelVariables for FmiBoolean {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.boolean.push(self);
    }
}

impl AppendToModelVariables for FmiString {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.string.push(self);
    }
}
