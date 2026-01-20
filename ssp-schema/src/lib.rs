use hard_xml::{XmlRead, XmlWrite};
use std::borrow::Cow;

#[cfg(false)]
mod tmp {
    /// Shared attributes for "model entities" (ssc:ABaseElement):
    /// - id: optional file-wide unique id
    /// - description: optional longer description
    /// (See SSC common attributes.)  [oai_citation:3‡ssp-standard.org](https://ssp-standard.org/docs/main/)
    #[derive(Clone, Debug, Default, PartialEq, XmlRead, XmlWrite)]
    pub struct BaseElementAttrs<'a> {
        #[xml(attr = "id")]
        pub id: Option<Cow<'a, str>>,

        #[xml(attr = "description")]
        pub description: Option<Cow<'a, str>>,
    }

    /// Shared top-level metadata attributes (SSC 4.3).  [oai_citation:4‡ssp-standard.org](https://ssp-standard.org/docs/main/)
    #[derive(Clone, Debug, Default, PartialEq, XmlRead, XmlWrite)]
    pub struct TopLevelAttrs<'a> {
        #[xml(attr = "author")]
        pub author: Option<Cow<'a, str>>,

        #[xml(attr = "fileversion")]
        pub fileversion: Option<Cow<'a, str>>,

        #[xml(attr = "copyright")]
        pub copyright: Option<Cow<'a, str>>,

        #[xml(attr = "license")]
        pub license: Option<Cow<'a, str>>,

        #[xml(attr = "generationTool")]
        pub generation_tool: Option<Cow<'a, str>>,

        #[xml(attr = "generationDateAndTime")]
        pub generation_date_and_time: Option<Cow<'a, str>>,
    }
}

/// Root element of an SSD file (ssd:SystemStructureDescription).
/// Sequence: System, Enumerations?, Units?, DefaultExperiment?,
///           (ssc:GMetaData)*, (ssc:GSignature)*, Annotations?
/// Attributes: version (required), name (required), plus common attrs.
///  [oai_citation:5‡GitHub](https://github.com/modelica/ssp-standard/blob/main/schema/SystemStructureDescription.xsd?utm_source=chatgpt.com)
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "SystemStructureDescription")]
pub struct SystemStructureDescription<'a> {
    // --- required attributes (SSD 5.1)  [oai_citation:6‡ssp-standard.org](https://ssp-standard.org/docs/main/)
    #[xml(attr = "version")]
    pub version: Cow<'a, str>, // spec says "1.0" or "2.0" (major.minor only)

    #[xml(attr = "name")]
    pub name: Cow<'a, str>,

    // --- common attribute groups (SSC 4.1 + 4.3)  [oai_citation:7‡ssp-standard.org](https://ssp-standard.org/docs/main/)
    #[xml(attr = "id")]
    pub id: Option<Cow<'a, str>>,
    #[xml(attr = "description")]
    pub description: Option<Cow<'a, str>>,

    #[xml(attr = "author")]
    pub author: Option<Cow<'a, str>>,
    #[xml(attr = "fileversion")]
    pub fileversion: Option<Cow<'a, str>>,
    #[xml(attr = "copyright")]
    pub copyright: Option<Cow<'a, str>>,
    #[xml(attr = "license")]
    pub license: Option<Cow<'a, str>>,
    #[xml(attr = "generationTool")]
    pub generation_tool: Option<Cow<'a, str>>,
    #[xml(attr = "generationDateAndTime")]
    pub generation_date_and_time: Option<Cow<'a, str>>,

    // --- children (SSD root sequence)  [oai_citation:8‡GitHub](https://github.com/modelica/ssp-standard/blob/main/schema/SystemStructureDescription.xsd?utm_source=chatgpt.com)
    #[xml(child = "System")]
    pub system: System<'a>, // placeholder for your full SSD system model

    #[xml(child = "Enumerations")]
    pub enumerations: Option<Enumerations<'a>>,

    #[xml(child = "Units")]
    pub units: Option<Units<'a>>,

    #[xml(child = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment<'a>>,

    // groups (0..n)
    #[xml(child = "MetaData")]
    pub meta_data: Vec<MetaData<'a>>,

    #[xml(child = "Signature")]
    pub signature: Vec<Signature<'a>>,

    // common child elements (SSC 4.2)
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations<'a>>,
}

/// Placeholder: you’d replace this with your fleshed-out SSD System model.
/// (SSD 5.3)  [oai_citation:9‡ssp-standard.org](https://ssp-standard.org/docs/main/)
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "System")]
pub struct System<'a> {
    #[xml(attr = "name")]
    pub name: Cow<'a, str>,

    // Common attrs frequently appear on “entities”
    #[xml(attr = "id")]
    pub id: Option<Cow<'a, str>>,
    #[xml(attr = "description")]
    pub description: Option<Cow<'a, str>>,

    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations<'a>>,
    // ... add Elements/Connectors/Connections/etc. here
}

/// SSC 4.2: Annotations container; must contain 1+ Annotation when present.  [oai_citation:10‡ssp-standard.org](https://ssp-standard.org/docs/main/)
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Annotations")]
pub struct Annotations<'a> {
    #[xml(child = "Annotation")]
    pub annotation: Vec<Annotation<'a>>,
}

/// Each Annotation has required `type` and “arbitrary XML content”.  [oai_citation:11‡ssp-standard.org](https://ssp-standard.org/docs/main/)
///
/// hard_xml does not support “any element subtree” fields, so this models only text content.
/// If you need full fidelity, parse the inner XML separately with another parser.
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Annotation")]
pub struct Annotation<'a> {
    #[xml(attr = "type")]
    pub ty: Cow<'a, str>,

    // Best-effort: text-only payload
    #[xml(text)]
    pub text: Cow<'a, str>,
}

/// SSC 4.4.1: Enumerations wrapper; contains definitions of Enumeration elements.  [oai_citation:12‡ssp-standard.org](https://ssp-standard.org/docs/main/)
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Enumerations")]
pub struct Enumerations<'a> {
    #[xml(child = "Enumeration")]
    pub enumeration: Vec<Enumeration<'a>>,
}

#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Enumeration")]
pub struct Enumeration<'a> {
    #[xml(attr = "name")]
    pub name: Cow<'a, str>,

    #[xml(child = "Item")]
    pub item: Vec<EnumerationItem<'a>>,
}

/// SSC 4.4.1.1: Enumeration Item has required name + value.  [oai_citation:13‡ssp-standard.org](https://ssp-standard.org/docs/main/)
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Item")]
pub struct EnumerationItem<'a> {
    #[xml(attr = "name")]
    pub name: Cow<'a, str>,

    #[xml(attr = "value")]
    pub value: i64,
}

/// SSC 4.4.2: Units wrapper; contains Unit elements.  [oai_citation:14‡ssp-standard.org](https://ssp-standard.org/docs/main/)
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Units")]
pub struct Units<'a> {
    #[xml(child = "Unit")]
    pub unit: Vec<Unit<'a>>,
}

#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Unit")]
pub struct Unit<'a> {
    #[xml(attr = "name")]
    pub name: Cow<'a, str>,

    #[xml(child = "BaseUnit")]
    pub base_unit: BaseUnit,
}

/// SSC 4.4.2.1: SI exponents plus optional factor/offset (defaults exist in spec).  [oai_citation:15‡ssp-standard.org](https://ssp-standard.org/docs/main/)
///
/// hard_xml can apply defaults via #[xml(default, ...)].
#[derive(Clone, Debug, Default, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "BaseUnit")]
pub struct BaseUnit {
    #[xml(default, attr = "kg")]
    pub kg: i32,
    #[xml(default, attr = "m")]
    pub m: i32,
    #[xml(default, attr = "s")]
    pub s: i32,
    #[xml(default, attr = "A")]
    pub a: i32,
    #[xml(default, attr = "K")]
    pub k: i32,
    #[xml(default, attr = "mol")]
    pub mol: i32,
    #[xml(default, attr = "cd")]
    pub cd: i32,
    #[xml(default, attr = "rad")]
    pub rad: i32,

    #[xml(default, attr = "factor")]
    pub factor: f64, // spec default 1  [oai_citation:16‡ssp-standard.org](https://ssp-standard.org/docs/main/)
    #[xml(default, attr = "offset")]
    pub offset: f64, // spec default 0  [oai_citation:17‡ssp-standard.org](https://ssp-standard.org/docs/main/)
}

/// SSD 5.1.1: DefaultExperiment has optional startTime / stopTime.  [oai_citation:18‡ssp-standard.org](https://ssp-standard.org/docs/main/)
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "DefaultExperiment")]
pub struct DefaultExperiment<'a> {
    #[xml(attr = "startTime")]
    pub start_time: Option<f64>,

    #[xml(attr = "stopTime")]
    pub stop_time: Option<f64>,

    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations<'a>>,
}

/// SSC 4.5.4: MetaData: kind/type/source/sourceBase + Content? + Signature*
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "MetaData")]
pub struct MetaData<'a> {
    #[xml(attr = "kind")]
    pub kind: Option<Cow<'a, str>>, // e.g. "general" or "quality"

    #[xml(attr = "type")]
    pub mime_type: Cow<'a, str>, // mandatory

    #[xml(attr = "source")]
    pub source: Option<Cow<'a, str>>,

    #[xml(attr = "sourceBase")]
    pub source_base: Option<Cow<'a, str>>,

    #[xml(child = "Content")]
    pub content: Option<MetaDataContent<'a>>,

    #[xml(child = "Signature")]
    pub signature: Vec<Signature<'a>>,
}

/// The spec allows inline XML payloads here.
/// This models text-only content for hard_xml convenience.
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Content")]
pub struct MetaDataContent<'a> {
    #[xml(text)]
    pub text: Cow<'a, str>,
}

/// SSC 4.5.5: Signature: role/type/source/sourceBase + Content?
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Signature")]
pub struct Signature<'a> {
    #[xml(attr = "role")]
    pub role: Cow<'a, str>, // "authenticity" | "suitability"

    #[xml(attr = "type")]
    pub mime_type: Cow<'a, str>, // mandatory

    #[xml(attr = "source")]
    pub source: Option<Cow<'a, str>>,

    #[xml(attr = "sourceBase")]
    pub source_base: Option<Cow<'a, str>>,

    #[xml(child = "Content")]
    pub content: Option<SignatureContent<'a>>,
}

/// Spec allows inline signature material.
/// Text-only approximation.
#[derive(Clone, Debug, PartialEq, XmlRead, XmlWrite)]
#[xml(tag = "Content")]
pub struct SignatureContent<'a> {
    #[xml(text)]
    pub text: Cow<'a, str>,
}
