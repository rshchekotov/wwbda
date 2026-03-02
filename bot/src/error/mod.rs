use std::fmt;

#[derive(Debug)]
pub enum DoomError {
    NotImplementedError { functionality: String },
}

impl std::error::Error for DoomError {}

impl fmt::Display for DoomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DoomError::NotImplementedError { functionality } => write!(
                f,
                "This functionality '{}' is not implemented.",
                functionality
            ),
        }
    }
}
