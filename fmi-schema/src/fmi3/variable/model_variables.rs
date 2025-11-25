use super::{
    AbstractVariableTrait, FmiBinary, FmiBoolean, FmiFloat32, FmiFloat64, FmiInt8, FmiInt16,
    FmiInt32, FmiInt64, FmiString, FmiUInt8, FmiUInt16, FmiUInt32, FmiUInt64,
    TypedArrayableVariableTrait,
};

#[derive(hard_xml::XmlRead, hard_xml::XmlWrite, Debug, PartialEq)]
pub enum Variable {
    #[xml(tag = "Int8")]
    Int8(FmiInt8),
    #[xml(tag = "UInt8")]
    UInt8(FmiUInt8),
    #[xml(tag = "Int16")]
    Int16(FmiInt16),
    #[xml(tag = "UInt16")]
    UInt16(FmiUInt16),
    #[xml(tag = "Int32")]
    Int32(FmiInt32),
    #[xml(tag = "UInt32")]
    UInt32(FmiUInt32),
    #[xml(tag = "Int64")]
    Int64(FmiInt64),
    #[xml(tag = "UInt64")]
    UInt64(FmiUInt64),
    #[xml(tag = "Float32")]
    Float32(FmiFloat32),
    #[xml(tag = "Float64")]
    Float64(FmiFloat64),
    #[xml(tag = "Boolean")]
    Boolean(FmiBoolean),
    #[xml(tag = "String")]
    String(FmiString),
    #[xml(tag = "Binary")]
    Binary(FmiBinary),
}

#[derive(Debug, PartialEq, Default, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ModelVariables")]
pub struct ModelVariables {
    #[xml(
        child = "Int8",
        child = "UInt8",
        child = "Int16",
        child = "UInt16",
        child = "Int32",
        child = "UInt32",
        child = "Int64",
        child = "UInt64",
        child = "Float32",
        child = "Float64",
        child = "Boolean",
        child = "String",
        child = "Binary"
    )]
    pub variables: Vec<Variable>,
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
        self.variables.iter().map(|v| match v {
            Variable::Int8(var) => var as &dyn AbstractVariableTrait,
            Variable::UInt8(var) => var as &dyn AbstractVariableTrait,
            Variable::Int16(var) => var as &dyn AbstractVariableTrait,
            Variable::UInt16(var) => var as &dyn AbstractVariableTrait,
            Variable::Int32(var) => var as &dyn AbstractVariableTrait,
            Variable::UInt32(var) => var as &dyn AbstractVariableTrait,
            Variable::Int64(var) => var as &dyn AbstractVariableTrait,
            Variable::UInt64(var) => var as &dyn AbstractVariableTrait,
            Variable::Float32(var) => var as &dyn AbstractVariableTrait,
            Variable::Float64(var) => var as &dyn AbstractVariableTrait,
            Variable::Boolean(var) => var as &dyn AbstractVariableTrait,
            Variable::String(var) => var as &dyn AbstractVariableTrait,
            Variable::Binary(var) => var as &dyn AbstractVariableTrait,
        })
    }

    /// Returns an iterator over all the float32 and float64 variables in the model description
    pub fn iter_floating(&self) -> impl Iterator<Item = &dyn TypedArrayableVariableTrait> {
        self.variables.iter().filter_map(|v| match v {
            Variable::Float32(var) => Some(var as &dyn TypedArrayableVariableTrait),
            Variable::Float64(var) => Some(var as &dyn TypedArrayableVariableTrait),
            _ => None,
        })
    }

    /// Finds a variable by its name.
    pub fn find_by_name(&self, name: &str) -> Option<&dyn AbstractVariableTrait> {
        self.iter_abstract().find(|v| v.name() == name)
    }

    /// Returns a vector of all Float32 variables
    pub fn float32(&self) -> Vec<&FmiFloat32> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Float32(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all Float64 variables
    pub fn float64(&self) -> Vec<&FmiFloat64> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Float64(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all Int8 variables
    pub fn int8(&self) -> Vec<&FmiInt8> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Int8(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all UInt8 variables
    pub fn uint8(&self) -> Vec<&FmiUInt8> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::UInt8(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all Int16 variables
    pub fn int16(&self) -> Vec<&FmiInt16> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Int16(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all UInt16 variables
    pub fn uint16(&self) -> Vec<&FmiUInt16> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::UInt16(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all Int32 variables
    pub fn int32(&self) -> Vec<&FmiInt32> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Int32(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all UInt32 variables
    pub fn uint32(&self) -> Vec<&FmiUInt32> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::UInt32(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all Int64 variables
    pub fn int64(&self) -> Vec<&FmiInt64> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Int64(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all UInt64 variables
    pub fn uint64(&self) -> Vec<&FmiUInt64> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::UInt64(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all Boolean variables
    pub fn boolean(&self) -> Vec<&FmiBoolean> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Boolean(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all String variables
    pub fn string(&self) -> Vec<&FmiString> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::String(var) => Some(var),
                _ => None,
            })
            .collect()
    }

    /// Returns a vector of all Binary variables
    pub fn binary(&self) -> Vec<&FmiBinary> {
        self.variables
            .iter()
            .filter_map(|v| match v {
                Variable::Binary(var) => Some(var),
                _ => None,
            })
            .collect()
    }
}

/// Append a variable to the given `ModelVariables` struct
pub trait AppendToModelVariables: AbstractVariableTrait {
    fn append_to_variables(self, variables: &mut ModelVariables);
}

impl AppendToModelVariables for FmiFloat32 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Float32(self));
    }
}

impl AppendToModelVariables for FmiFloat64 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Float64(self));
    }
}

impl AppendToModelVariables for FmiInt8 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Int8(self));
    }
}

impl AppendToModelVariables for FmiInt16 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Int16(self));
    }
}

impl AppendToModelVariables for FmiInt32 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Int32(self));
    }
}

impl AppendToModelVariables for FmiInt64 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Int64(self));
    }
}

impl AppendToModelVariables for FmiUInt8 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::UInt8(self));
    }
}

impl AppendToModelVariables for FmiUInt16 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::UInt16(self));
    }
}

impl AppendToModelVariables for FmiUInt32 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::UInt32(self));
    }
}

impl AppendToModelVariables for FmiUInt64 {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::UInt64(self));
    }
}

impl AppendToModelVariables for FmiBoolean {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Boolean(self));
    }
}

impl AppendToModelVariables for FmiString {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::String(self));
    }
}

impl AppendToModelVariables for FmiBinary {
    fn append_to_variables(self, variables: &mut ModelVariables) {
        variables.variables.push(Variable::Binary(self));
    }
}
