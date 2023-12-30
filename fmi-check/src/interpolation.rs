use arrow::{array::PrimitiveArray, datatypes::ArrowPrimitiveType};
use num_traits::{Num, NumCast};

/// Index and interval fraction
#[derive(Debug, PartialEq)]
pub struct PreLookup(usize, f64);

/// Calculates the index and interval fraction that specify how its input value time relates to the breakpoint dataset.
///
/// Assumes that array is sorted in ascending order. Values outside the range of the array are clamped to the first or last value.
pub fn pre_lookup<T>(array: &PrimitiveArray<T>, value: T::Native) -> PreLookup
where
    T: ArrowPrimitiveType,
    T::Native: NumCast,
{
    let found = array
        .values()
        .binary_search_by(|t| t.partial_cmp(&value).unwrap());

    match found {
        // value is exactly on a breakpoint
        Ok(index) => PreLookup(index, 0.0),
        // `index` is the index of the first breakpoint that is greater than value
        Err(index) => {
            if index == 0 {
                // value is before the first breakpoint
                PreLookup(0, 0.0)
            } else if index == array.len() {
                // value is after the last breakpoint
                PreLookup(index - 1, 1.0)
            } else {
                // value is between two breakpoints
                let t0: f64 = NumCast::from(array.value(index - 1)).unwrap();
                let t1: f64 = NumCast::from(array.value(index)).unwrap();
                let value: f64 = NumCast::from(value).unwrap();
                let fraction = (value - t0) / (t1 - t0);
                PreLookup(index - 1, fraction)
            }
        }
    }
}

/// Interpolates the value of the array at the specified pre-lookup index and interval fraction.
pub fn interpolate<T>(
    array: &PrimitiveArray<T>,
    PreLookup(index, fraction): &PreLookup,
) -> T::Native
where
    T: ArrowPrimitiveType,
    T::Native: NumCast,
{
    let t0: f64 = NumCast::from(array.value(*index)).unwrap();
    let t1: f64 = NumCast::from(array.value(*index + 1)).unwrap();
    NumCast::from(t0 + fraction * (t1 - t0)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::{interpolate, pre_lookup, PreLookup};
    use arrow::{array::PrimitiveArray, datatypes::Int32Type};

    #[test]
    fn test_pre_lookup() {
        let array = PrimitiveArray::from(vec![0.0, 1.0, 2.0, 3.0, 4.0]);

        assert_eq!(pre_lookup(&array, -1.0), PreLookup(0, 0.0));
        assert_eq!(pre_lookup(&array, 0.0), PreLookup(0, 0.0));
        assert_eq!(pre_lookup(&array, 0.5), PreLookup(0, 0.5));
        assert_eq!(pre_lookup(&array, 1.0), PreLookup(1, 0.0));
        assert_eq!(pre_lookup(&array, 1.5), PreLookup(1, 0.5));
        assert_eq!(pre_lookup(&array, 2.0), PreLookup(2, 0.0));
        assert_eq!(pre_lookup(&array, 2.5), PreLookup(2, 0.5));
        assert_eq!(pre_lookup(&array, 3.0), PreLookup(3, 0.0));
        assert_eq!(pre_lookup(&array, 3.5), PreLookup(3, 0.5));
        assert_eq!(pre_lookup(&array, 4.0), PreLookup(4, 0.0));
        assert_eq!(pre_lookup(&array, 5.0), PreLookup(4, 1.0));
    }

    #[test]
    fn test_interpolation() {
        let time = PrimitiveArray::from(vec![0.0, 2.0, 3.0]);
        let value1 = PrimitiveArray::from(vec![0.0, 2.0, 4.0]);
        let value2 = PrimitiveArray::<Int32Type>::from(vec![1, 3, 5]);

        assert_eq!(interpolate(&value1, &pre_lookup(&time, 0.0)), 0.0);
        assert_eq!(interpolate(&value1, &pre_lookup(&time, 1.0)), 1.0);
        assert_eq!(interpolate(&value2, &pre_lookup(&time, 1.0)), 2);
        assert_eq!(interpolate(&value1, &pre_lookup(&time, 1.5)), 1.5);
        assert_eq!(interpolate(&value2, &pre_lookup(&time, 1.5)), 2);
    }
}
