use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, parse_quote};

use crate::model::{Field, FieldAttributeOuter, Model};

pub struct ModelGetSetImpl<'a> {
    pub struct_name: &'a Ident,
    pub model: &'a Model,
}

/// Check if a field has the skip attribute set to true
fn has_skip_attribute(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(var_attr) if var_attr.skip))
}

/// Filter out fields that have the skip attribute
fn filter_non_skipped_fields(fields: &[Field]) -> Vec<&Field> {
    fields
        .iter()
        .filter(|field| !has_skip_attribute(field))
        .collect()
}

fn build_getter_fn(
    fn_name: Ident,
    ty: syn::Type,
    model: &crate::model::Model,
) -> proc_macro2::TokenStream {
    // Filter out skipped fields
    let non_skipped_fields = filter_non_skipped_fields(&model.fields);

    // Create scalar count variables:
    // for each field, `let field_name_count = <FieldType as ModelGetSet<Self>>::FIELD_COUNT;`
    let scalar_var_counts = non_skipped_fields.iter().map(|f| {
        let count_name = format_ident!("{}_count", f.ident);
        let field_type = &f.rust_type;
        quote! {
            let #count_name = <#field_type as ::fmi_export::fmi3::ModelGetSet<M>>::FIELD_COUNT as u32;
        }
    });

    // Generate if-else conditions to route to the correct field
    // We need to compute cumulative sums like f0_count, f0_count + inner_count, etc.
    let mut conditions = Vec::new();

    for (i, field) in non_skipped_fields.iter().enumerate() {
        let field_name = &field.ident;
        let field_type = &field.rust_type;
        let count_name = format_ident!("{}_count", field.ident);

        // Build cumulative sum for the condition
        let cumulative_sum = if i == 0 {
            quote! { #count_name }
        } else {
            let prev_sums: Vec<_> = non_skipped_fields
                .iter()
                .take(i)
                .map(|f| format_ident!("{}_count", f.ident))
                .collect();
            quote! { #(#prev_sums)+* + #count_name }
        };

        // Build vr offset for the field call
        let vr_offset = if i == 0 {
            quote! { vr }
        } else {
            let prev_sums: Vec<_> = non_skipped_fields
                .iter()
                .take(i)
                .map(|f| format_ident!("{}_count", f.ident))
                .collect();
            quote! { vr - (#(#prev_sums)+*) }
        };

        conditions.push(quote! {
            if vr < #cumulative_sum {
                <#field_type as ::fmi_export::fmi3::ModelGetSet<M>>::#fn_name(&self.#field_name, #vr_offset, values, context)
            }
        });
    }

    // Chain all conditions together with else if
    let chained_conditions = if conditions.is_empty() {
        quote! { Err(::fmi::fmi3::Fmi3Error::Error) }
    } else {
        let mut result = quote! { { Err(::fmi::fmi3::Fmi3Error::Error) } };
        for condition in conditions.into_iter().rev() {
            result = quote! { #condition else #result };
        }
        result
    };

    quote! {
        fn #fn_name(
            &self,
            vr: ::fmi::fmi3::binding::fmi3ValueReference,
            values: &mut [#ty],
            context: &dyn ::fmi_export::fmi3::Context<M>
        ) -> Result<usize, ::fmi::fmi3::Fmi3Error> {
            #(#scalar_var_counts)*
            #chained_conditions
        }
    }
}

fn build_clock_get_fn(model: &crate::model::Model) -> proc_macro2::TokenStream {
    // Filter out skipped fields
    let non_skipped_fields = filter_non_skipped_fields(&model.fields);

    // Create scalar count variables:
    let scalar_var_counts = non_skipped_fields.iter().map(|f| {
        let count_name = format_ident!("{}_count", f.ident);
        let field_type = &f.rust_type;
        quote! {
            let #count_name = <#field_type as ::fmi_export::fmi3::ModelGetSet<M>>::FIELD_COUNT as u32;
        }
    });

    // Generate if-else conditions to route to the correct field
    let mut conditions = Vec::new();

    for (i, field) in non_skipped_fields.iter().enumerate() {
        let field_name = &field.ident;
        let field_type = &field.rust_type;
        let count_name = format_ident!("{}_count", field.ident);

        // Check if this field has Output causality for Clock variables
        let has_output_causality = field.attrs.iter().any(|attr| {
            if let crate::model::FieldAttributeOuter::Variable(var_attr) = attr {
                if let Some(causality) = &var_attr.causality {
                    matches!(causality.0, fmi::fmi3::schema::Causality::Output)
                } else {
                    false
                }
            } else {
                false
            }
        });

        // Build cumulative sum for the condition
        let cumulative_sum = if i == 0 {
            quote! { #count_name }
        } else {
            let prev_sums: Vec<_> = non_skipped_fields
                .iter()
                .take(i)
                .map(|f| format_ident!("{}_count", f.ident))
                .collect();
            quote! { #(#prev_sums)+* + #count_name }
        };

        // Build vr offset for the field call
        let vr_offset = if i == 0 {
            quote! { vr }
        } else {
            let prev_sums: Vec<_> = non_skipped_fields
                .iter()
                .take(i)
                .map(|f| format_ident!("{}_count", f.ident))
                .collect();
            quote! { vr - (#(#prev_sums)+*) }
        };

        if has_output_causality {
            conditions.push(quote! {
                if vr < #cumulative_sum {
                    <#field_type as ::fmi_export::fmi3::ModelGetSet<M>>::get_clock(&mut self.#field_name, #vr_offset, value, context)
                }
            });
        } else {
            // For non-Output clocks: return an error
            conditions.push(quote! {
                if vr < #cumulative_sum {
                    Err(::fmi::fmi3::Fmi3Error::Error) // get_clock only valid for Output causality
                }
            });
        }
    }

    // Chain all conditions together with else if
    let chained_conditions = if conditions.is_empty() {
        quote! { Err(::fmi::fmi3::Fmi3Error::Error) }
    } else {
        let mut result = quote! { { Err(::fmi::fmi3::Fmi3Error::Error) } };
        for condition in conditions.into_iter().rev() {
            result = quote! { #condition else #result };
        }
        result
    };

    quote! {
        fn get_clock(
            &mut self,
            vr: ::fmi::fmi3::binding::fmi3ValueReference,
            value: &mut ::fmi::fmi3::binding::fmi3Clock,
            context: &dyn ::fmi_export::fmi3::Context<M>
        ) -> Result<(), ::fmi::fmi3::Fmi3Error> {
            #(#scalar_var_counts)*
            #chained_conditions
        }
    }
}

fn build_clock_set_fn(model: &crate::model::Model) -> proc_macro2::TokenStream {
    // Filter out skipped fields
    let non_skipped_fields = filter_non_skipped_fields(&model.fields);

    // Create scalar count variables:
    let scalar_var_counts = non_skipped_fields.iter().map(|f| {
        let count_name = format_ident!("{}_count", f.ident);
        let field_type = &f.rust_type;
        quote! {
            let #count_name = <#field_type as ::fmi_export::fmi3::ModelGetSet<M>>::FIELD_COUNT as u32;
        }
    });

    // Generate if-else conditions to route to the correct field
    let mut conditions = Vec::new();

    for (i, field) in non_skipped_fields.iter().enumerate() {
        let field_name = &field.ident;
        let field_type = &field.rust_type;
        let count_name = format_ident!("{}_count", field.ident);

        // Check if this field has Input causality for Clock variables
        let has_input_causality = field.attrs.iter().any(|attr| {
            if let crate::model::FieldAttributeOuter::Variable(var_attr) = attr {
                if let Some(causality) = &var_attr.causality {
                    matches!(causality.0, fmi::fmi3::schema::Causality::Input)
                } else {
                    false
                }
            } else {
                false
            }
        });

        // Build cumulative sum for the condition
        let cumulative_sum = if i == 0 {
            quote! { #count_name }
        } else {
            let prev_sums: Vec<_> = non_skipped_fields
                .iter()
                .take(i)
                .map(|f| format_ident!("{}_count", f.ident))
                .collect();
            quote! { #(#prev_sums)+* + #count_name }
        };

        // Build vr offset for the field call
        let vr_offset = if i == 0 {
            quote! { vr }
        } else {
            let prev_sums: Vec<_> = non_skipped_fields
                .iter()
                .take(i)
                .map(|f| format_ident!("{}_count", f.ident))
                .collect();
            quote! { vr - (#(#prev_sums)+*) }
        };

        if has_input_causality {
            // For Input clocks: allow setting
            conditions.push(quote! {
                if vr < #cumulative_sum {
                    <#field_type as ::fmi_export::fmi3::ModelGetSet<M>>::set_clock(&mut self.#field_name, #vr_offset, value, context)
                }
            });
        } else {
            // For non-Input clocks: return an error
            conditions.push(quote! {
                if vr < #cumulative_sum {
                    Err(::fmi::fmi3::Fmi3Error::Error) // set_clock only valid for Input causality
                }
            });
        }
    }

    // Chain all conditions together with else if
    let chained_conditions = if conditions.is_empty() {
        quote! { Err(::fmi::fmi3::Fmi3Error::Error) }
    } else {
        let mut result = quote! { { Err(::fmi::fmi3::Fmi3Error::Error) } };
        for condition in conditions.into_iter().rev() {
            result = quote! { #condition else #result };
        }
        result
    };

    quote! {
        fn set_clock(
            &mut self,
            vr: ::fmi::fmi3::binding::fmi3ValueReference,
            value: &::fmi::fmi3::binding::fmi3Clock,
            context: &dyn ::fmi_export::fmi3::Context<M>
        ) -> Result<(), ::fmi::fmi3::Fmi3Error> {
            #(#scalar_var_counts)*
            #chained_conditions
        }
    }
}

impl ToTokens for ModelGetSetImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;

        // Filter out skipped fields for the FIELD_COUNT calculation
        let non_skipped_fields = filter_non_skipped_fields(&self.model.fields);
        let field_types = non_skipped_fields.iter().map(|f| &f.rust_type);

        // Generate all getter/setter functions
        let boolean_get_fn =
            build_getter_fn(format_ident!("get_boolean"), parse_quote!(bool), self.model);
        let float32_get_fn =
            build_getter_fn(format_ident!("get_float32"), parse_quote!(f32), self.model);
        let float64_get_fn =
            build_getter_fn(format_ident!("get_float64"), parse_quote!(f64), self.model);
        let int8_get_fn = build_getter_fn(format_ident!("get_int8"), parse_quote!(i8), self.model);
        let int16_get_fn =
            build_getter_fn(format_ident!("get_int16"), parse_quote!(i16), self.model);
        let int32_get_fn =
            build_getter_fn(format_ident!("get_int32"), parse_quote!(i32), self.model);
        let int64_get_fn =
            build_getter_fn(format_ident!("get_int64"), parse_quote!(i64), self.model);
        let uint8_get_fn =
            build_getter_fn(format_ident!("get_uint8"), parse_quote!(u8), self.model);
        let uint16_get_fn =
            build_getter_fn(format_ident!("get_uint16"), parse_quote!(u16), self.model);
        let uint32_get_fn =
            build_getter_fn(format_ident!("get_uint32"), parse_quote!(u32), self.model);
        let uint64_get_fn =
            build_getter_fn(format_ident!("get_uint64"), parse_quote!(u64), self.model);
        let string_get_fn = build_getter_fn(
            format_ident!("get_string"),
            parse_quote!(std::ffi::CString),
            self.model,
        );

        // Generate Clock-specific methods
        let clock_get_fn = build_clock_get_fn(self.model);
        let clock_set_fn = build_clock_set_fn(self.model);

        tokens.extend(quote! {
            impl<M: ::fmi_export::fmi3::Model> ::fmi_export::fmi3::ModelGetSet<M> for #struct_name {
                const FIELD_COUNT: usize = #(
                    <#field_types as ::fmi_export::fmi3::ModelGetSet<M>>::FIELD_COUNT
                )+*;

                #boolean_get_fn
                #float32_get_fn
                #float64_get_fn
                #int8_get_fn
                #int16_get_fn
                #int32_get_fn
                #int64_get_fn
                #uint8_get_fn
                #uint16_get_fn
                #uint32_get_fn
                #uint64_get_fn
                #string_get_fn
                #clock_get_fn
                #clock_set_fn
            }
        });
    }
}
