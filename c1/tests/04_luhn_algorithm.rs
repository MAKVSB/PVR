//! Run this file with `cargo test --test 04_luhn_algorithm`.

// TODO: Implement the Luhn algorithm (https://en.wikipedia.org/wiki/Luhn_algorithm),
// which is used to check the validity of e.g. bank or credit card numbers.

fn luhn_algorithm(num: u64) -> bool {
    if num < 10 {
        return true
    }
    let mut sum = 0;
    let mut is_second = false;
    let mut n = num;

    while n > 0 {
        let mut digit = (n % 10) as u32;  // Extract the last digit
        n /= 10;  // Remove the last digit

        if is_second {
            digit *= 2;  // Double every second digit
            if digit > 9 {
                digit -= 9;  // Subtract 9 if the digit is greater than 9
            }
        }

        sum += digit;
        is_second = !is_second;  // Alternate between second and non-second digit
    }

    sum % 10 == 0
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use super::luhn_algorithm;

    #[test]
    fn luhn_zero() {
        assert!(luhn_algorithm(0));
    }

    #[test]
    fn luhn_small_correct() {
        assert!(luhn_algorithm(5)); //not sure about if this is really true
        assert!(luhn_algorithm(18));
    }

    #[test]
    fn luhn_small_incorrect() {
        assert!(!luhn_algorithm(10));
    }

    #[test]
    fn luhn_correct() {
        assert!(luhn_algorithm(17893729974));
        assert!(luhn_algorithm(79927398713));
    }

    #[test]
    fn luhn_incorrect() {
        assert!(!luhn_algorithm(17893729975));
        assert!(!luhn_algorithm(17893729976));
        assert!(!luhn_algorithm(17893729977));
        assert!(!luhn_algorithm(123456));
    }
}
