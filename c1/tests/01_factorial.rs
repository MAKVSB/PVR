//! Run this file with `cargo test --test 01_factorial`.

fn factorial(a: u32) -> u32 {
    if a == 0 {
        1
    } else {
        factorial(a-1) * a
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use super::factorial;

    #[test]
    fn factorial_0() {
        assert_eq!(factorial(0), 1);
    }

    #[test]
    fn factorial_1() {
        assert_eq!(factorial(1), 1);
    }

    #[test]
    fn factorial_2() {
        assert_eq!(factorial(2), 2);
    }

    #[test]
    fn factorial_5() {
        assert_eq!(factorial(5), 120);
    }
}
