use fmi::fmi3::{Fmi3Error, Fmi3Res, GetSet, binding};

use crate::fmi3::Model;

/// Macro to generate getter implementations for ModelInstance
macro_rules! instance_getter {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            fn [<get_ $name>](
                &mut self,
                vrs: &[Self::ValueRef],
                values: &mut [$ty],
            ) -> Result<Fmi3Res, Fmi3Error> {
                if self.is_dirty_values {
                    self.model.calculate_values(&self.context)?;
                    self.is_dirty_values = false;
                }
                self.model.[<get_ $name>](vrs, values, &self.context)
            }
        }
    };
}

/// Macro to generate setter implementations for ModelInstance
macro_rules! instance_setter {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            fn [<set_ $name>](
                &mut self,
                vrs: &[Self::ValueRef],
                values: &[$ty],
            ) -> Result<Fmi3Res, Fmi3Error> {
                // Validate variable setting restrictions before setting values
                for &vr in vrs {
                    self.validate_variable_setting(vr)?;
                }

                self.model.[<set_ $name>](vrs, values, &self.context)?;
                self.is_dirty_values = true;
                Ok(Fmi3Res::OK)
            }
        }
    };
}

/// Macro to generate both getter and setter for standard types
macro_rules! instance_getter_setter {
    ($name:ident, $ty:ty) => {
        instance_getter!($name, $ty);
        instance_setter!($name, $ty);
    };
}

/// Macro for special getter/setter pairs with different return types
macro_rules! instance_getter_setter_special {
    (string) => {
        fn get_string(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &mut [std::ffi::CString],
        ) -> Result<(), Fmi3Error> {
            self.model.get_string(vrs, values, &self.context)
        }

        fn set_string(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &[std::ffi::CString],
        ) -> Result<(), Fmi3Error> {
            // Validate variable setting restrictions before setting values
            for &vr in vrs {
                self.validate_variable_setting(vr)?;
            }

            self.model.set_string(vrs, values, &self.context)?;
            self.is_dirty_values = true;
            Ok(())
        }
    };
    (binary) => {
        fn get_binary(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &mut [&mut [u8]],
        ) -> Result<Vec<usize>, Fmi3Error> {
            self.model.get_binary(vrs, values, &self.context)
        }

        fn set_binary(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &[&[u8]],
        ) -> Result<(), Fmi3Error> {
            // Validate variable setting restrictions before setting values
            for &vr in vrs {
                self.validate_variable_setting(vr)?;
            }

            self.model.set_binary(vrs, values, &self.context)?;
            self.is_dirty_values = true;
            Ok(())
        }
    };
}

impl<F> GetSet for super::ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    type ValueRef = binding::fmi3ValueReference;

    // Standard getter/setter pairs
    instance_getter_setter!(boolean, bool);
    instance_getter_setter!(float32, f32);
    instance_getter_setter!(float64, f64);
    instance_getter_setter!(int8, i8);
    instance_getter_setter!(int16, i16);
    instance_getter_setter!(int32, i32);
    instance_getter_setter!(int64, i64);
    instance_getter_setter!(uint8, u8);
    instance_getter_setter!(uint16, u16);
    instance_getter_setter!(uint32, u32);
    instance_getter_setter!(uint64, u64);

    // Special getter/setter pairs with different signatures
    instance_getter_setter_special!(string);
    instance_getter_setter_special!(binary);
}
