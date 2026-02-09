use std::sync::Arc;

use arrow::{
    array::{ArrayRef, StringArray},
    datatypes::{DataType, Field, Fields, Schema},
};
use fmi::{
    fmi3::{import::Fmi3Import, schema::Causality},
    schema::fmi3::{Dimension, Variability, VariableType},
    traits::FmiImport,
};
use fmi::schema::fmi3::{Annotations, ResolvedTerminal, resolve_terminals};

use crate::sim::{
    io::StartValues,
    output::{OutputDimension, OutputKind, TerminalFieldBinding, TerminalSchemaBinding},
    traits::ImportSchemaBuilder,
};

impl ImportSchemaBuilder for Fmi3Import
where
    Self::ValueRef: From<u32>,
{
    fn inputs_schema(&self) -> Schema {
        let input_fields = self
            .model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == Causality::Input)
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
            .filter(|v| v.causality() == Causality::Output)
            .map(|v| Field::new(v.name(), v.data_type().into(), false))
            .chain(std::iter::once(time))
            .collect::<Fields>();

        Schema::new(output_fields)
    }

    fn continuous_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> + '_ {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| {
                v.causality() == Causality::Input
                    && v.variability() == fmi::fmi3::schema::Variability::Continuous
            })
            .map(|v| {
                (
                    Field::new(v.name(), v.data_type().into(), false),
                    v.value_reference(),
                )
            })
    }

    fn discrete_inputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> + '_ {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| {
                v.causality() == Causality::Input
                    && (v.variability() == Variability::Discrete
                        || v.variability() == Variability::Tunable)
            })
            .map(|v| {
                (
                    Field::new(v.name(), v.data_type().into(), false),
                    v.value_reference(),
                )
            })
    }

    fn outputs(&self) -> impl Iterator<Item = (Field, Self::ValueRef)> {
        self.model_description()
            .model_variables
            .iter_abstract()
            .filter(|v| v.causality() == Causality::Output)
            .map(|v| {
                (
                    Field::new(v.name(), v.data_type().into(), false),
                    v.value_reference(),
                )
            })
    }

    fn output_field_for_vr(&self, vr: Self::ValueRef) -> anyhow::Result<Field> {
        let var = self
            .model_description()
            .model_variables
            .find_by_value_reference(vr)
            .ok_or_else(|| anyhow::anyhow!("Output VR {} not found", vr))?;

        if var.causality() != Causality::Output {
            return Err(anyhow::anyhow!("VR {} is not an Output variable", vr));
        }

        let name = var.name();
        let desc = var.description();
        let elem_type = var.data_type();
        let dimensions = self
            .model_description()
            .model_variables
            .dimensions_by_value_reference(vr)
            .unwrap_or(&[]);

        let dtype = if dimensions.is_empty() {
            DataType::from(elem_type)
        } else if let Some(len) = fixed_array_len(dimensions) {
            let item_field = Field::new("item", DataType::from(elem_type), false);
            DataType::FixedSizeList(Arc::new(item_field), len as i32)
        } else {
            let item_field = Field::new("item", DataType::from(elem_type), false);
            DataType::List(Arc::new(item_field))
        };

        let mut field = Field::new(name, dtype, false);
        if let Some(desc) = desc {
            let mut meta = field.metadata().clone();
            meta.insert("description".to_string(), desc.to_string());
            field = field.with_metadata(meta);
        }
        Ok(field)
    }

    fn output_kind_for_vr(&self, vr: Self::ValueRef) -> anyhow::Result<OutputKind> {
        let var = self
            .model_description()
            .model_variables
            .find_by_value_reference(vr)
            .ok_or_else(|| anyhow::anyhow!("Output VR {} not found", vr))?;

        if var.causality() != Causality::Output {
            return Err(anyhow::anyhow!("VR {} is not an Output variable", vr));
        }

        let kind = match var.data_type() {
            VariableType::FmiFloat32 => OutputKind::Float32,
            VariableType::FmiFloat64 => OutputKind::Float64,
            VariableType::FmiInt8 => OutputKind::Int8,
            VariableType::FmiUInt8 => OutputKind::UInt8,
            VariableType::FmiInt16 => OutputKind::Int16,
            VariableType::FmiUInt16 => OutputKind::UInt16,
            VariableType::FmiInt32 => OutputKind::Int32,
            VariableType::FmiUInt32 => OutputKind::UInt32,
            VariableType::FmiInt64 => OutputKind::Int64,
            VariableType::FmiUInt64 => OutputKind::UInt64,
            VariableType::FmiBoolean => OutputKind::Boolean,
            VariableType::FmiString => OutputKind::Utf8,
            VariableType::FmiBinary => OutputKind::Binary,
            VariableType::FmiClock => OutputKind::Clock,
        };
        Ok(kind)
    }

    fn output_array_dims_for_vr(
        &self,
        vr: Self::ValueRef,
    ) -> anyhow::Result<Vec<OutputDimension<Self::ValueRef>>> {
        let var = self
            .model_description()
            .model_variables
            .find_by_value_reference(vr)
            .ok_or_else(|| anyhow::anyhow!("Output VR {} not found", vr))?;

        if var.causality() != Causality::Output {
            return Err(anyhow::anyhow!("VR {} is not an Output variable", vr));
        }

        let dims = self
            .model_description()
            .model_variables
            .dimensions_by_value_reference(vr)
            .unwrap_or(&[]);
        if dims.is_empty() {
            return Ok(Vec::new());
        }
        let mapped = dims
            .iter()
            .map(|dim| match dim {
                Dimension::Fixed(size) => OutputDimension::Fixed(*size as usize),
                Dimension::Variable(vr) => OutputDimension::Variable(Self::ValueRef::from(*vr)),
            })
            .collect();
        Ok(mapped)
    }

    fn output_terminal_bindings(
        &self,
    ) -> anyhow::Result<Vec<TerminalSchemaBinding<Self::ValueRef>>> {
        let Some(terminals) = self.terminals_and_icons() else {
            return Ok(Vec::new());
        };
        let resolved = resolve_terminals(terminals, self.model_description())
            .map_err(|err| anyhow::anyhow!("Failed to resolve terminals: {:?}", err))?;

        let mut bindings = Vec::new();
        for terminal in resolved.terminals {
            collect_terminal_bindings::<Self::ValueRef>(&terminal, &mut bindings);
        }
        Ok(bindings)
    }
    fn parse_start_values(
        &self,
        start_values: &[String],
    ) -> anyhow::Result<StartValues<Self::ValueRef>> {
        let mut structural_parameters: Vec<(Self::ValueRef, ArrayRef)> = vec![];
        let mut variables: Vec<(Self::ValueRef, ArrayRef)> = vec![];

        for start_value in start_values {
            let (name, value) = start_value
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("Invalid start value"))?;

            let var = self
                .model_description()
                .model_variables
                .iter_abstract()
                .find(|v| v.name() == name)
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Invalid variable name: {name}. Valid variables are: {valid:?}",
                        valid = self
                            .model_description()
                            .model_variables
                            .iter_abstract()
                            .map(|v| v.name())
                            .collect::<Vec<_>>()
                    )
                })?;

            let dt = arrow::datatypes::DataType::from(var.data_type());
            let ary = StringArray::from(vec![value.to_string()]);
            let ary = arrow::compute::cast(&ary, &dt)
                .map_err(|e| anyhow::anyhow!("Error casting type: {e}"))?;

            if var.causality() == Causality::StructuralParameter {
                structural_parameters.push((var.value_reference(), ary));
            } else {
                variables.push((var.value_reference(), ary));
            }
        }

        Ok(StartValues {
            structural_parameters,
            variables,
        })
    }

    fn binary_max_size(&self, vr: Self::ValueRef) -> Option<usize> {
        self.model_description()
            .model_variables
            .binary_max_size(vr)
            .map(|m| m as usize)
    }
}

fn collect_terminal_bindings<VR: From<u32> + Copy>(
    terminal: &ResolvedTerminal<'_>,
    out: &mut Vec<TerminalSchemaBinding<VR>>,
) {
    if let Some(binding) = terminal_binding_from_terminal::<VR>(terminal) {
        out.push(binding);
    }
    for nested in &terminal.terminals {
        collect_terminal_bindings::<VR>(nested, out);
    }
}

fn terminal_binding_from_terminal<VR: From<u32> + Copy>(
    terminal: &ResolvedTerminal<'_>,
) -> Option<TerminalSchemaBinding<VR>> {
    let schema_name = annotation_value(terminal.terminal.annotations.as_ref(), "foxglove.schema")
        .or_else(|| terminal.terminal.terminal_kind.clone());
    let schema_url = annotation_value(
        terminal.terminal.annotations.as_ref(),
        "foxglove.schema_url",
    );
    if schema_name.is_none() && schema_url.is_none() {
        return None;
    }

    let mut fields = Vec::new();
    for member in &terminal.members {
        if member.variable.causality() != Causality::Output {
            continue;
        }
        let field_name = annotation_value(member.member.annotations.as_ref(), "foxglove.field")
            .or_else(|| member.member.member_name.clone())
            .unwrap_or_else(|| member.variable.name().to_string());
        fields.push(TerminalFieldBinding {
            vr: VR::from(member.variable.value_reference()),
            field_name,
        });
    }
    if fields.is_empty() {
        return None;
    }

    Some(TerminalSchemaBinding {
        terminal_name: terminal.terminal.name.clone(),
        schema_name,
        schema_url,
        fields,
    })
}

fn annotation_value(annotations: Option<&Annotations>, key: &str) -> Option<String> {
    annotations.and_then(|annotations| {
        annotations
            .annotations
            .iter()
            .find(|annotation| annotation.r#type == key)
            .map(|annotation| annotation.content.clone())
    })
}

fn fixed_array_len(dimensions: &[Dimension]) -> Option<u64> {
    dimensions.iter().try_fold(1u64, |acc, dim| match dim {
        Dimension::Fixed(size) => acc.checked_mul(*size),
        Dimension::Variable(_) => None,
    })
}
