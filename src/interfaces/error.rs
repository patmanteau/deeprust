use std::fmt;

type Result<T> = std::result::Result<T, InterfaceError>;

#[derive(Debug)]
pub enum InterfaceError {
    Invalid,
    ParseFen,
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InterfaceError::EmptyVec =>
                write!(f, "please use a vector with at least one element"),
            // This is a wrapper, so defer to the underlying types' implementation of `fmt`.
            InterfaceError::Parse(ref e) => e.fmt(f),
        }
    }
}
