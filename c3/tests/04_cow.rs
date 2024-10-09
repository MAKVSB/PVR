//! Run this file with `cargo test --test 04_cow`.

// TODO: Implement a function called `to_upper_if_needed`, which takes a string slice
// and returns the uppercase version of that string.
// If the string was already uppercase, it should not perform any allocations!

enum OwnedOrRef<'a> {
    Owned(String),
    Borrowed(&'a str),
}

fn to_upper_if_needed<'a>(string: &'a str) -> OwnedOrRef<'a> {
    if string.chars().all(|c| c.is_uppercase()) {
        return OwnedOrRef::Borrowed(string);
    }
    OwnedOrRef::Owned(string.to_uppercase())
}
