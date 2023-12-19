use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{Float32Attributes, Float64Attributes, RealBaseAttributes};

pub trait BaseTypeTrait {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct TypeDefinitionBase {
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(attribute)]
    pub description: Option<String>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float32Type {
    #[yaserde(flatten)]
    pub base: TypeDefinitionBase,
    #[yaserde(flatten)]
    pub base_attr: RealBaseAttributes,
    #[yaserde(flatten)]
    pub attr: Float32Attributes,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float64Type {
    #[yaserde(flatten)]
    pub base: TypeDefinitionBase,
    #[yaserde(flatten)]
    pub base_attr: RealBaseAttributes,
    #[yaserde(flatten)]
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
