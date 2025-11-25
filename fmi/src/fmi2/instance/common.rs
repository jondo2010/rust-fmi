use std::ffi::CStr;

use crate::fmi2::{Fmi2Error, Fmi2Res, Fmi2Status, binding};
use crate::traits::{FmiStatus, InstanceTag};

use super::{Common, Instance};

impl<Tag: InstanceTag> Common for Instance<Tag> {
    fn get_version(&self) -> &str {
        // Safety: The FMI API guarantees that the pointer is valid within the lifetime of the FMU
        unsafe { CStr::from_ptr(self.binding.fmi2GetVersion()) }
            .to_str()
            .expect("Error converting string")
    }

    fn get_types_platform(&self) -> &str {
        // Safety: The FMI API guarantees that the pointer is valid within the lifetime of the FMU
        unsafe { CStr::from_ptr(self.binding.fmi2GetTypesPlatform()) }
            .to_str()
            .expect("Error converting string")
    }

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi2Res, Fmi2Error> {
        let category_cstr = categories
            .iter()
            .map(|c| std::ffi::CString::new(*c).expect("Error building CString"))
            .collect::<Vec<_>>();

        let category_ptrs: Vec<_> = category_cstr.iter().map(|c| c.as_ptr()).collect();

        Fmi2Status::from(unsafe {
            self.binding.fmi2SetDebugLogging(
                self.component,
                logging_on as binding::fmi2Boolean,
                category_ptrs.len(),
                category_ptrs.as_ptr(),
            )
        })
        .ok()
    }

    fn setup_experiment(
        &mut self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding.fmi2SetupExperiment(
                self.component,
                tolerance.is_some() as binding::fmi2Boolean,
                tolerance.unwrap_or(0.0),
                start_time,
                stop_time.is_some() as binding::fmi2Boolean,
                stop_time.unwrap_or(0.0),
            )
        })
        .ok()
    }

    fn enter_initialization_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2EnterInitializationMode(self.component) }).ok()
    }

    fn exit_initialization_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2ExitInitializationMode(self.component) }).ok()
    }

    fn reset(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2Reset(self.component) }).ok()
    }

    fn terminate(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2Terminate(self.component) }).ok()
    }

    fn get_real(
        &mut self,
        vrs: &[binding::fmi2ValueReference],
        values: &mut [binding::fmi2Real],
    ) -> Result<Fmi2Res, Fmi2Error> {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status::from(unsafe {
            self.binding
                .fmi2GetReal(self.component, vrs.as_ptr(), vrs.len(), values.as_mut_ptr())
        })
        .ok()
    }

    fn get_integer(
        &mut self,
        vrs: &[binding::fmi2ValueReference],
        values: &mut [binding::fmi2Integer],
    ) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding.fmi2GetInteger(
                self.component,
                vrs.as_ptr(),
                vrs.len(),
                values.as_mut_ptr(),
            )
        })
        .ok()
    }

    fn get_boolean(
        &mut self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [binding::fmi2Boolean],
    ) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding
                .fmi2GetBoolean(self.component, sv.as_ptr(), sv.len(), v.as_mut_ptr())
        })
        .ok()
    }

    fn get_string(
        &mut self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [std::ffi::CString],
    ) -> Result<(), Fmi2Error> {
        let n_values = v.len();

        // Create an array of null pointers to receive the string pointers from FMI
        let mut value_ptrs: Vec<binding::fmi2String> = vec![std::ptr::null(); n_values];

        // Call the FMI function to get string pointers
        let status = unsafe {
            self.binding.fmi2GetString(
                self.component,
                sv.as_ptr(),
                sv.len(),
                value_ptrs.as_mut_ptr(),
            )
        };

        Fmi2Status::from(status).ok()?;

        // Copy the C strings into the output CString values
        for (value, ptr) in v.iter_mut().zip(value_ptrs.iter()) {
            if ptr.is_null() {
                return Err(Fmi2Error::Error);
            }
            let cstr = unsafe { std::ffi::CStr::from_ptr(*ptr) };
            // Copy the string data into the provided CString
            *value = std::ffi::CString::new(cstr.to_bytes()).map_err(|_| Fmi2Error::Error)?;
        }

        Ok(())
    }

    fn set_real(
        &mut self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Real],
    ) -> Result<Fmi2Res, Fmi2Error> {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status::from(unsafe {
            self.binding
                .fmi2SetReal(self.component, vrs.as_ptr(), values.len(), values.as_ptr())
        })
        .ok()
    }

    fn set_integer(
        &mut self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Integer],
    ) -> Result<Fmi2Res, Fmi2Error> {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status::from(unsafe {
            self.binding
                .fmi2SetInteger(self.component, vrs.as_ptr(), values.len(), values.as_ptr())
        })
        .ok()
    }

    fn set_boolean(
        &mut self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Boolean],
    ) -> Result<Fmi2Res, Fmi2Error> {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status::from(unsafe {
            self.binding
                .fmi2SetBoolean(self.component, vrs.as_ptr(), values.len(), values.as_ptr())
        })
        .ok()
    }

    fn set_string(
        &mut self,
        vrs: &[binding::fmi2ValueReference],
        values: &[std::ffi::CString],
    ) -> Result<(), Fmi2Error> {
        let ptrs = values
            .iter()
            .map(|s| s.as_c_str().as_ptr())
            .collect::<Vec<_>>();

        let status = unsafe {
            self.binding
                .fmi2SetString(self.component, vrs.as_ptr(), vrs.len() as _, ptrs.as_ptr())
        };

        Fmi2Status::from(status).ok()?;
        Ok(())
    }

    fn get_directional_derivative(
        &self,
        unknown_vrs: &[binding::fmi2ValueReference],
        known_vrs: &[binding::fmi2ValueReference],
        dv_known_values: &[binding::fmi2Real],
        dv_unknown_values: &mut [binding::fmi2Real],
    ) -> Result<Fmi2Res, Fmi2Error> {
        assert!(unknown_vrs.len() == dv_unknown_values.len());
        assert!(known_vrs.len() == dv_unknown_values.len());
        Fmi2Status::from(unsafe {
            self.binding.fmi2GetDirectionalDerivative(
                self.component,
                unknown_vrs.as_ptr(),
                unknown_vrs.len(),
                known_vrs.as_ptr(),
                known_vrs.len(),
                dv_known_values.as_ptr(),
                dv_unknown_values.as_mut_ptr(),
            )
        })
        .ok()
    }

    #[cfg(false)]
    fn set_values(&mut self, vrs: &[binding::fmi2ValueReference], values: &arrow::array::ArrayRef) {
        use arrow::datatypes::DataType;
        match values.data_type() {
            DataType::Boolean => {
                let values: arrow::array::Int32Array =
                    arrow::compute::cast(values, &DataType::Int32)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_boolean(vrs, values.values());
            }
            DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32 => {
                let values: arrow::array::Int32Array =
                    arrow::compute::cast(values, &DataType::Int32)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_integer(vrs, values.values());
            }
            DataType::Float32 | DataType::Float64 => {
                let values: arrow::array::Float64Array =
                    arrow::compute::cast(values, &DataType::Float64)
                        .map(|a| arrow::array::downcast_array(&a))
                        .expect("Error casting");
                self.set_real(vrs, values.values());
            }
            DataType::Binary => todo!(),
            DataType::Utf8 => {
                let values: arrow::array::StringArray = arrow::array::downcast_array(values);
                let strings = values
                    .into_iter()
                    .map(|s| CString::new(s.unwrap_or_default()))
                    .collect::<Result<Vec<_>, _>>()
                    .expect("Error converting string");
                let values: Vec<_> = strings.iter().map(|s| s.as_ptr()).collect();
                self.set_string(vrs, &values);
            }
            _ => unimplemented!("Unsupported data type"),
        }
    }
}
