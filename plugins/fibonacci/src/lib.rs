use std::mem::replace;

use interface::plugin;
use num_bigint::BigUint;
use num_traits::{One, Zero};

plugin! { name: "fibonacci"; main: fibonacci }

/// Naive implementation of Fibonacci numbers.
fn fibonacci(n: u64) -> String {
    let mut f0: BigUint = Zero::zero();
    let mut f1: BigUint = One::one();
    for _ in 0..n {
        let f2 = f0 + &f1;
        f0 = replace(&mut f1, f2);
    }
    f0.to_string()
}
