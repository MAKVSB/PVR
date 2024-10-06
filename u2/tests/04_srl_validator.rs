//! Run this file with `cargo test --test 04_srl_validator`.

// TODO: Implement a SRL (Simple Resource Locator) validator.
// A SRL consists of two parts, an optional protocol (string) and an address (string).
// The format of the SRL looks like this: `[<protocol>://]<address>`
// The protocol and the address have to contain only lowercase English characters.
// Protocol must not be empty if :// is present in the SRL.
// Address must not be empty.
//
// As an example, these are valid SRLs:
// - `http://foo`
// - `bar://baz`
// - `foobar`
//
// And these are invalid SRLs:
// - `http://foo1` (invalid character in address)
// - `asd://bar://` (invalid character in address)
// - `://baz` (empty protocol)
// - `01://baz` (invalid character in protocol)
//
// Create a struct `SRL` in a module named `srl`. Expose functions for parsing a SRL and getting
// its individual parts, but do not allow modifying the fields of `SRL` outside its module.
// Do not use regular expressions, SRLs can be easily parsed with a big of parsing logic.
//
// Hint: Put `#[derive(Debug, Eq, PartialEq)]` on top of `SRL` and `SRLValidationError`,
// so that asserts in tests work.

mod srl {
    #[derive(Debug, Eq, PartialEq)]
    pub struct SRL {
        protocol: Option<String>,
        address: String,
    }

    impl SRL {
        pub fn get_protocol(&self) -> Option<&str> {
            self.protocol.as_deref()
        }

        pub fn get_address(&self) -> &str {
            &self.address
        }

        pub fn new(input: &str) -> Result<Self, SRLValidationError> {
            let parts: Vec<&str> = input.splitn(2, "://").collect();

            match parts.len() {
                1 => {
                    // no protocol
                    let address = parts[0];

                    if address.is_empty() {
                        return Err(SRLValidationError::EmptyAddress);
                    }
                    match address.chars().find(|c| !c.is_ascii_lowercase()) {
                        Some(c) => Err(SRLValidationError::InvalidCharacterInAddress(c)),
                        None => Ok(SRL {
                            protocol: None,
                            address: address.to_string(),
                        }),
                    }
                }
                _ => {
                    // with protocol
                    let protocol = parts[0];
                    let address = parts[1];

                    if protocol.is_empty() {
                        return Err(SRLValidationError::EmptyProtocol);
                    }
                    if let Some(c) = protocol.chars().find(|c| !c.is_ascii_lowercase()) {
                        return Err(SRLValidationError::InvalidCharacterInProtocol(c));
                    }

                    if address.is_empty() {
                        return Err(SRLValidationError::EmptyAddress);
                    }
                    if let Some(c) = address.chars().find(|c| !c.is_ascii_lowercase()) {
                        return Err(SRLValidationError::InvalidCharacterInAddress(c));
                    }

                    Ok(SRL {
                        protocol: Some(protocol.to_string()),
                        address: address.to_string(),
                    })
                }
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum SRLValidationError {
        InvalidCharacterInAddress(char),
        InvalidCharacterInProtocol(char),
        EmptyAddress,
        EmptyProtocol,
    }
}

/// Below you can find a set of unit tests.
#[cfg(test)]
mod tests {
    use super::srl::{SRLValidationError, SRL};

    #[test]
    fn empty_address() {
        assert_eq!(SRL::new(""), Err(SRLValidationError::EmptyAddress));
    }

    #[test]
    fn only_separator() {
        assert_eq!(SRL::new("://"), Err(SRLValidationError::EmptyProtocol));
    }

    #[test]
    fn empty_protocol() {
        assert_eq!(SRL::new("://foo"), Err(SRLValidationError::EmptyProtocol));
    }

    #[test]
    fn multiple_protocols() {
        assert_eq!(
            SRL::new("ab://bc://foo"),
            Err(SRLValidationError::InvalidCharacterInAddress(':'))
        );
    }

    #[test]
    fn invalid_protocol() {
        assert_eq!(
            SRL::new("bAc://foo"),
            Err(SRLValidationError::InvalidCharacterInProtocol('A'))
        );
        assert_eq!(
            SRL::new("a02://foo"),
            Err(SRLValidationError::InvalidCharacterInProtocol('0'))
        );
    }

    #[test]
    fn invalid_address_with_protocol() {
        assert_eq!(
            SRL::new("abc://fo1o"),
            Err(SRLValidationError::InvalidCharacterInAddress('1'))
        );
        assert_eq!(
            SRL::new("bar://fooABc"),
            Err(SRLValidationError::InvalidCharacterInAddress('A'))
        );
    }

    #[test]
    fn invalid_address_without_protocol() {
        assert_eq!(
            SRL::new("fo1o"),
            Err(SRLValidationError::InvalidCharacterInAddress('1'))
        );
        assert_eq!(
            SRL::new("fooABc"),
            Err(SRLValidationError::InvalidCharacterInAddress('A'))
        );
    }

    #[test]
    fn invalid_protocol_and_address() {
        assert_eq!(
            SRL::new("bAc://fo2o"),
            Err(SRLValidationError::InvalidCharacterInProtocol('A'))
        );
        assert_eq!(
            SRL::new("a02://barBAZ"),
            Err(SRLValidationError::InvalidCharacterInProtocol('0'))
        );
    }

    #[test]
    fn no_protocol() {
        let srl = SRL::new("foobar").unwrap();
        assert_eq!(srl.get_protocol(), None);
        assert_eq!(srl.get_address(), "foobar");
    }

    #[test]
    fn protocol_and_scheme() {
        let srl = SRL::new("bar://foobar").unwrap();
        assert_eq!(srl.get_protocol(), Some("bar"));
        assert_eq!(srl.get_address(), "foobar");
    }
}
