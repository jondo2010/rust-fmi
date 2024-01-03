//! Interpolation of breakpoint data.

use arrow::{array::PrimitiveArray, datatypes::ArrowPrimitiveType};
use num_traits::NumCast;

/// Index and interval fraction
#[derive(Debug, PartialEq)]
pub struct PreLookup(usize, f64);

impl PreLookup {
    /// Calculates the index and interval fraction that specify how its input value time relates to
    /// the breakpoint dataset.
    ///
    /// Assumes that array is sorted in ascending order. Values outside the range of the array are
    /// clamped to the first or last value.
    pub fn new<T>(array: &PrimitiveArray<T>, value: T::Native) -> Self
    where
        T: ArrowPrimitiveType,
        T::Native: NumCast,
    {
        let found = array
            .values()
            .binary_search_by(|t| t.partial_cmp(&value).unwrap());

        match found {
            // value is exactly on a breakpoint
            Ok(index) => Self(index, 0.0),
            // `index` is the index of the first breakpoint that is greater than value
            Err(index) => {
                if index == 0 {
                    // value is before the first breakpoint
                    Self(0, 0.0)
                } else if index == array.len() {
                    // value is after the last breakpoint
                    Self(index - 1, 1.0)
                } else {
                    // value is between two breakpoints
                    let t0: f64 = NumCast::from(array.value(index - 1)).unwrap();
                    let t1: f64 = NumCast::from(array.value(index)).unwrap();
                    let value: f64 = NumCast::from(value).unwrap();
                    let fraction = (value - t0) / (t1 - t0);
                    Self(index - 1, fraction)
                }
            }
        }
    }

    /// Interpolates the value of the array at the pre-lookup index and interval fraction.
    pub fn interpolate<T, A>(&self, array: &PrimitiveArray<T>) -> T::Native
    where
        T: ArrowPrimitiveType,
        T::Native: NumCast,
        A: Interpolate,
    {
        A::interpolate(self, array)
    }
}

pub trait Interpolate {
    fn interpolate<T>(pre: &PreLookup, array: &PrimitiveArray<T>) -> T::Native
    where
        T: ArrowPrimitiveType,
        T::Native: NumCast;
}

/// Disables interpolation and returns the table value corresponding to the breakpoint closest to
/// the input. If the input is equidistant from two adjacent breakpoints, the breakpoint with the
/// higher index is chosen.
pub struct Nearest;
impl Interpolate for Nearest {
    fn interpolate<T>(pre: &PreLookup, array: &PrimitiveArray<T>) -> T::Native
    where
        T: ArrowPrimitiveType,
        T::Native: NumCast,
    {
        let (index, fraction) = (pre.0, pre.1);
        let index = if fraction < 0.5 { index } else { index + 1 }.min(array.len() - 1);
        array.value(index)
    }
}

/// Fits a line between the adjacent breakpoints, and returns the point on that line corresponding
/// to the input.
pub struct Linear;
impl Interpolate for Linear {
    fn interpolate<T>(pre: &PreLookup, array: &PrimitiveArray<T>) -> T::Native
    where
        T: ArrowPrimitiveType,
        T::Native: NumCast,
    {
        let (index, fraction) = (pre.0.min(array.len() - 1), pre.1);
        let t0: f64 = NumCast::from(array.value(index)).unwrap();
        let t1: f64 = NumCast::from(array.value(index + 1)).unwrap();
        NumCast::from(t0 + fraction * (t1 - t0)).unwrap()
    }
}

/// Fits a cubic spline to the adjacent breakpoints, and returns the point on that spline
/// corresponding to the input.
pub struct Cubic;
impl Interpolate for Cubic {
    fn interpolate<T>(pre: &PreLookup, array: &PrimitiveArray<T>) -> T::Native
    where
        T: ArrowPrimitiveType,
        T::Native: NumCast,
    {
        let (index, fraction) = (pre.0.min(array.len() - 1), pre.1);
        let t0: f64 = NumCast::from(array.value(index)).unwrap();
        let t1: f64 = NumCast::from(array.value(index + 1)).unwrap();
        let t2: f64 = NumCast::from(array.value(index + 2)).unwrap();
        let t3: f64 = NumCast::from(array.value(index + 3)).unwrap();

        let a = -0.5 * t0 + 1.5 * t1 - 1.5 * t2 + 0.5 * t3;
        let b = t0 - 2.5 * t1 + 2.0 * t2 - 0.5 * t3;
        let c = -0.5 * t0 + 0.5 * t2;
        let d = t1;

        NumCast::from(a * fraction.powi(3) + b * fraction.powi(2) + c * fraction + d).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::{Interpolate, Linear, PreLookup};
    use arrow::{array::PrimitiveArray, datatypes::Int32Type};

    #[test]
    fn test_pre_lookup() {
        let array = PrimitiveArray::from(vec![0.0, 1.0, 2.0, 3.0, 4.0]);

        assert_eq!(PreLookup::new(&array, -1.0), PreLookup(0, 0.0));
        assert_eq!(PreLookup::new(&array, 0.0), PreLookup(0, 0.0));
        assert_eq!(PreLookup::new(&array, 0.5), PreLookup(0, 0.5));
        assert_eq!(PreLookup::new(&array, 1.0), PreLookup(1, 0.0));
        assert_eq!(PreLookup::new(&array, 1.5), PreLookup(1, 0.5));
        assert_eq!(PreLookup::new(&array, 2.0), PreLookup(2, 0.0));
        assert_eq!(PreLookup::new(&array, 2.5), PreLookup(2, 0.5));
        assert_eq!(PreLookup::new(&array, 3.0), PreLookup(3, 0.0));
        assert_eq!(PreLookup::new(&array, 3.5), PreLookup(3, 0.5));
        assert_eq!(PreLookup::new(&array, 4.0), PreLookup(4, 0.0));
        assert_eq!(PreLookup::new(&array, 5.0), PreLookup(4, 1.0));
    }

    #[test]
    fn test_interpolation() {
        let time = PrimitiveArray::from(vec![0.0, 2.0, 3.0]);
        let value1 = PrimitiveArray::from(vec![0.0, 2.0, 4.0]);
        let value2 = PrimitiveArray::<Int32Type>::from(vec![1, 3, 5]);

        let pl0 = PreLookup::new(&time, 0.0);
        let pl1 = PreLookup::new(&time, 1.0);
        let pl15 = PreLookup::new(&time, 1.5);

        assert_eq!(Linear::interpolate(&pl0, &value1), 0.0);
        assert_eq!(Linear::interpolate(&pl0, &value2), 1);
        assert_eq!(Linear::interpolate(&pl1, &value1), 1.0);
        assert_eq!(Linear::interpolate(&pl1, &value2), 2);
        assert_eq!(Linear::interpolate(&pl15, &value1), 1.5);
        assert_eq!(Linear::interpolate(&pl15, &value2), 2);
    }
}
