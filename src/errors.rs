use std::{error::Error, fmt::Display};

/// errors for generating a create3 salt.
#[derive(Debug, PartialEq)]
pub enum Create3GenerateSaltError {
    /// prefix is too long (max 20 bytes).
    PrefixTooLong,
    /// prefix is not hex encoded.
    PrefixNotHexEncoded,
}

impl Error for Create3GenerateSaltError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl Display for Create3GenerateSaltError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Create3GenerateSaltError::PrefixTooLong => {
                "prefix too long (max 20 bytes)."
            }
            Create3GenerateSaltError::PrefixNotHexEncoded => {
                "prefix not hex encoded."
            }
        })
    }
}
