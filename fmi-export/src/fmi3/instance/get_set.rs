use fmi::fmi3::{Fmi3Error, Fmi3Res, GetSet, binding};

use crate::fmi3::Model;

/// Macro to generate getter implementations for ModelInstance
macro_rules! instance_getter {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            fn [<get_ $name>](
                &mut self,
                vrs: &[binding::fmi3ValueReference],
                values: &mut [$ty],
            ) -> Result<Fmi3Res, Fmi3Error> {
                if self.is_dirty_values {
                    self.model.calculate_values(&self.context)?;
                    self.is_dirty_values = false;
                }
                let mut value_index = 0;
                for vr in vrs.iter() {
                    let vr = F::ValueRef::try_from(*vr)?;
                    let elements_read = self.model.[<get_ $name>](vr, &mut values[value_index..], &self.context)?;
                    value_index += elements_read;
                }
                Ok(Fmi3Res::OK)
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
                vrs: &[binding::fmi3ValueReference],
                values: &[$ty],
            ) -> Result<Fmi3Res, Fmi3Error> {
                // Validate variable setting restrictions before setting values
                let mut value_index = 0;
                for vr in vrs.iter() {
                    self.validate_variable_setting(*vr)?;
                    let vr = F::ValueRef::try_from(*vr)?;
                    let elements_written = self.model.[<set_ $name>](vr, &values[value_index..], &self.context)?;
                    value_index += elements_written;
                }

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

impl<F> GetSet for super::ModelInstance<F>
where
    F: Model,
{
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

    fn get_string(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &mut [std::ffi::CString],
    ) -> Result<(), Fmi3Error> {
        let mut value_index = 0;
        for vr in vrs.iter() {
            let vr = F::ValueRef::try_from(*vr)?;
            let elements_read = self.model.get_string(vr, &mut values[value_index..], &self.context)?;
            value_index += elements_read;
        }
        Ok(())
    }

    fn set_string(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &[std::ffi::CString],
    ) -> Result<(), Fmi3Error> {
        let mut value_index = 0;
        for vr in vrs.iter() {
            self.validate_variable_setting(*vr)?;
            let vr = F::ValueRef::try_from(*vr)?;
            let elements_written = self.model.set_string(vr, &values[value_index..], &self.context)?;
            value_index += elements_written;
        }
        self.is_dirty_values = true;
        Ok(())
    }

    fn get_binary(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &mut [&mut [u8]],
    ) -> Result<Vec<usize>, Fmi3Error> {
        let mut result_sizes = Vec::new();
        let mut value_index = 0;
        for vr in vrs.iter() {
            let vr = F::ValueRef::try_from(*vr)?;
            let binary_sizes = self.model.get_binary(vr, &mut values[value_index..], &self.context)?;
            result_sizes.extend(binary_sizes.iter());
            value_index += binary_sizes.len();
        }
        Ok(result_sizes)
    }

    fn set_binary(
        &mut self,
        vrs: &[binding::fmi3ValueReference],
        values: &[&[u8]],
    ) -> Result<(), Fmi3Error> {
        let mut value_index = 0;
        for vr in vrs.iter() {
            self.validate_variable_setting(*vr)?;
            let vr = F::ValueRef::try_from(*vr)?;
            let elements_written = self.model.set_binary(vr, &values[value_index..], &self.context)?;
            value_index += elements_written;
        }
        self.is_dirty_values = true;
        Ok(())
    }

    fn get_clock(
        &mut self,
        _vrs: &[binding::fmi3ValueReference],
        _values: &mut [binding::fmi3Clock],
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Default implementation: clocks not supported
        Err(Fmi3Error::Error)
    }

    fn set_clock(
        &mut self,
        _vrs: &[binding::fmi3ValueReference],
        _values: &[binding::fmi3Clock],
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Default implementation: clocks not supported
        Err(Fmi3Error::Error)
    }
}
