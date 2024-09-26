//! Run this file with `cargo test --test 06_write_tests`.

/// This function implements a string sanitization logic that should uphold the following
/// properties:
/// - After sanitization, the result must not end with the character `x`
/// - After sanitization, the result must not end with the character `o`
/// - After sanitization, the result must not end with the string `.exe`
///
/// The function assumes that the input to the function only consists of lower and uppercase
/// characters from the English alphabet and digits 0-9.
///
/// The implementation contains some bugs.
///
/// Your task is to write a set (at least 8) of unit tests, use them to find (at least 2) bugs in
/// this function and then fix the function.
fn sanitize(input: &str) -> &str {
    let mut sanitized = input;
    
    loop {
        let mut trimmed = sanitized;
        if trimmed.ends_with(".exe") {
            trimmed = &trimmed[0..trimmed.len() - 4];
        }

        trimmed = trimmed.trim_end_matches('x');
        trimmed = trimmed.trim_end_matches('o');

        if trimmed.len() == sanitized.len() {
            sanitized = trimmed;
            break;
        }

        
        sanitized = trimmed;
    }
    sanitized
}

///
/// Bonus: can you find any bugs using the [proptest](https://proptest-rs.github.io/proptest/intro.html)
/// crate?
/// Note that you will probably need to run `cargo test` with the `PROPTEST_DISABLE_FAILURE_PERSISTENCE=1`
/// environment variable to make proptest work for tests stored in the `tests` directory.
#[cfg(test)]
mod tests {
    use super::sanitize;

    #[test]
    fn match_empty_string() {
        assert_eq!(sanitize(""), "");
    }

    #[test]
    fn match_remove_trailing_x() {
        assert_eq!(sanitize("TotallyValidInputx"), "TotallyValidInput");
    }

    #[test]
    fn match_remove_trailing_0() {
        assert_eq!(sanitize("TotallyValidInputo"), "TotallyValidInput");
    }

    #[test]
    fn match_remove_trailing_exe() {
        assert_eq!(sanitize("TotallyValidInput.exe"), "TotallyValidInput");
    }


    #[test]
    fn match_remove_trailing_xo() {
        // Problem 1 => It deleted firstly x then o. Not in reverse order
        assert_eq!(sanitize("TotallyValidInputxo"), "TotallyValidInput");
    }


    #[test]
    fn match_remove_trailing_ox() {
        assert_eq!(sanitize("TotallyValidInputox"), "TotallyValidInput");
    }

    #[test]
    fn match_remove_trailing_oxox() {
        // Problem 1 => It can also be multiple times together with not in order
        assert_eq!(sanitize("TotallyValidInputoxox"), "TotallyValidInput");
    }

    #[test]
    fn match_remove_trailing_ox_exe() {
        // Problem 2 => Exe should be removed first. Then check x or os
        assert_eq!(sanitize("TotallyValidInputox.exe"), "TotallyValidInput");
    }























}
