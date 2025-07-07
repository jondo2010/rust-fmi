use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{Float32Attributes, Float64Attributes, RealBaseAttributes};

pub trait BaseTypeTrait {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct TypeDefinitionBase {
    #[yaserde(attribute = true)]
    pub name: String,
    #[yaserde(attribute = true)]
    pub description: Option<String>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float32Type {
    #[yaserde(flatten = true)]
    pub base: TypeDefinitionBase,
    #[yaserde(flatten = true)]
    pub base_attr: RealBaseAttributes,
    #[yaserde(flatten = true)]
    pub attr: Float32Attributes,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float64Type {
    #[yaserde(flatten = true)]
    pub base: TypeDefinitionBase,
    #[yaserde(flatten = true)]
    pub base_attr: RealBaseAttributes,
    #[yaserde(flatten = true)]
    pub attr: Float64Attributes,
}

impl BaseTypeTrait for Float32Type {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> Option<&str> {
        self.base.description.as_deref()
    }
}

impl BaseTypeTrait for Float64Type {
    fn name(&self) -> &str {
        &self.base.name
    }

    fn description(&self) -> Option<&str> {
        self.base.description.as_deref()
    }
}
