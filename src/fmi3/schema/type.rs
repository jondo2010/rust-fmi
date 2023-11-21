use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{Float32Attributes, Float64Attributes, RealBaseAttributes};

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct TypeDefinitionBase {
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(attribute)]
    pub description: Option<String>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float32Type {
    #[yaserde(flatten)]
    pub base: TypeDefinitionBase,
    #[yaserde(flatten)]
    pub base_attr: RealBaseAttributes,
    #[yaserde(flatten)]
    pub attr: Float32Attributes,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float64Type {
    #[yaserde(flatten)]
    pub base: TypeDefinitionBase,
    #[yaserde(flatten)]
    pub base_attr: RealBaseAttributes,
    #[yaserde(flatten)]
    pub attr: Float64Attributes,
}
