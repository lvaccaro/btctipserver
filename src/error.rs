use std::fmt;

#[derive(Debug)]
pub enum Error {
    Sled(bdk::sled::Error),
    Bdk(bdk::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Sled(e) => write!(f, "{}", e),
            Error::Bdk(e) => write!(f, "{}", e),
        }
    }
}

macro_rules! impl_error {
    ( $from:ty, $to:ident ) => {
        impl std::convert::From<$from> for Error {
            fn from(err: $from) -> Self {
                Error::$to(err)
            }
        }
    };
}

impl_error!(bdk::sled::Error, Sled);
impl_error!(bdk::Error, Bdk);
