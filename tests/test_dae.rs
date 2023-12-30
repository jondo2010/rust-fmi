// Import the necessary libraries
use ndarray::{Array1, Array2, ScalarOperand};
use ndarray_linalg::solve::Solve;
use ndarray_linalg::types::Scalar;

// Define the DAE solver function
#[cfg(feature = "disabled")]
fn dae_solver<F, G, S>(f: F, g: G, y0: Array1<S>, t0: S, tf: S, h: S) -> Array2<S>
where
    F: Fn(S, Array1<S>) -> Array1<S>,
    G: Fn(S, Array1<S>) -> Array2<S>,
    S: Scalar + ScalarOperand + Copy + std::cmp::PartialOrd,
{
    let mut t = t0;
    let mut y = y0;
    let mut res = Array2::zeros((1, y.len()));
    res.row_mut(0).assign(&y);

    while t < tf {
        let k1 = f(t, y.clone());
        let k2 = f(
            t + h / S::from(2.0).unwrap(),
            y.clone() + h * k1 / S::from(2.0).unwrap(),
        );
        let k3 = f(
            t + h / S::from(2.0).unwrap(),
            y.clone() + h * k2 / S::from(2.0).unwrap(),
        );
        let k4 = f(t + h, y.clone() + h * k3);

        let k = (k1 + S::from(2.0).unwrap() * k2 + S::from(2.0).unwrap() * k3 + k4)
            / S::from(6.0).unwrap();
        let y_next = y.clone() + h * k;
        let g_next = g(t + h, y_next.clone());

        let mut y_guess = y_next.clone();
        let mut y_diff = y_guess.clone() - y_next.clone();
        let mut iter = 0;

        while y_diff.norm_l2() > S::from(1e-6).unwrap() && iter < 100 {
            let jacobian = g(t + h, y_guess.clone()).jacobian(&y_guess);
            let delta_y = jacobian.solve(&(-g_next));
            y_guess = y_guess + delta_y;
            y_diff = y_guess.clone() - y_next.clone();
            iter += 1;
        }

        y = y_guess;
        t = t + h;
        res = ndarray::stack![
            ndarray::Axis(0),
            res,
            y.clone().insert_axis(ndarray::Axis(0))
        ];
    }

    res
}

// Define the DAE system
fn f<S: Scalar + Copy>(t: S, y: Array1<S>) -> Array1<S> {
    let mut res = Array1::zeros(y.len());
    res[0] = y[1];
    res[1] = -S::from(9.81).unwrap() * y[0];
    res
}

fn g<S: Scalar + Copy>(t: S, y: Array1<S>) -> Array2<S> {
    let mut res = Array2::zeros((y.len(), y.len()));
    res[(0, 1)] = S::one();
    res[(1, 0)] = -S::from(9.81).unwrap();
    res
}

// Define the main function
#[test]
fn main() {
    type S = f64;
    let y0 = Array1::from(vec![1.0, 0.0]);
    let t0 = 0.0;
    let tf = 10.0;
    let h = 0.01;

    // let res = dae_solver(f, g, y0, t0, tf, h);
    // println!("{:?}", res);
}
