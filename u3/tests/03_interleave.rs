//! Run this file with `cargo test --test 03_interleave`.

// TODO: Implement a function called `interleave`, which will take two string slices and return
// a string that contains the characters from the two input strings interleaved.
// The first character of the output should start with the first character of the first argument.
// See tests for details.
//
// Can you write the function without any explicit indexing (`str[index]`)?
//
// Hint: you can use `string.chars()` to create an iterator over the Unicode characters of a string.

use std::fmt::Write;

fn interleave(str1: &str, str2: &str) -> String {
    let str1_chars = str1.chars();
    let str2_chars = str2.chars();

    // Clippy forced me this way. And im not even mad
    let mut result_chars = str1_chars.clone()
        .zip(str2_chars.clone())
        .fold(String::new(), | mut acc, (c1, c2)| {
            let _ = write!(acc, "{}{}", c1, c2);
            acc
        });

    // Im kindof proud of this one. I have no idea how i came up with this.
    match str1.len().cmp(&str2.len()) {
        std::cmp::Ordering::Less => {
            result_chars.push_str(str2_chars.skip(str1_chars.count()).collect::<String>().as_str());
        },
        std::cmp::Ordering::Greater => {
            result_chars.push_str(str1_chars.skip(str2_chars.count()).collect::<String>().as_str());
        },
        std::cmp::Ordering::Equal => {},
    }

    result_chars
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use crate::interleave;

    #[test]
    fn interleave_empty() {
        assert_eq!(interleave("", ""), "");
    }

    #[test]
    fn interleave_only_left() {
        assert_eq!(interleave("a", ""), "a");
        assert_eq!(interleave("zxjas", ""), "zxjas");
    }

    #[test]
    fn interleave_only_right() {
        assert_eq!(interleave("", "z"), "z");
        assert_eq!(interleave("", "foobar"), "foobar");
    }

    #[test]
    fn interleave_same_length() {
        assert_eq!(interleave("abcdef", "012345"), "a0b1c2d3e4f5");
    }

    #[test]
    fn interleave_first_longer() {
        assert_eq!(
            interleave("Programming Rust", "O'Reilly"),
            "POr'oRgerialmlmying Rust"
        );
    }

    #[test]
    fn interleave_second_longer() {
        assert_eq!(
            interleave("ahoj, jak se máš?", "díky za optání, mám se dobře"),
            "adhíokjy,  zjaa ko psteá nmíá,š ?mám se dobře"
        );
    }
}
