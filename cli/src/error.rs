use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum InvalidInputError {
    InputLength,
    NonAscii(u8),
}

impl Error for InvalidInputError {}

impl fmt::Display for InvalidInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputLength => {
                write!(f, "Guess must have 5 characters.")?;
            }
            Self::NonAscii(invalid_char) => {
                write!(f, "expected ascii, found: {:#x}", invalid_char)?;
            }
        }

        Ok(())
    }
}
