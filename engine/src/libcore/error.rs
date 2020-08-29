use tokio::time::Error as TimeError;
use std::io::ErrorKind;
use serde::ser::SerializeStructVariant;

#[derive(Debug)]
pub enum Error {
    TimeCacheError(TimeError),
    IO(std::io::Error),
    RangeError,
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Self::IO(x)
    }
}

impl From<TimeError> for Error {
    fn from(x: TimeError) -> Self {
        Self::TimeCacheError(x)
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

use serde::{Serialize, Serializer, ser::SerializeStruct};

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 3 is the number of fields in the struct.
        match self {
            Error::IO(e) => {
                match e.kind() {
                    ErrorKind::Other => {
                        e.raw_os_error();
                        
                        let mut sv = serializer.serialize_struct_variant("Error", 0, "IO", 1)?;

                        //state.serialize_struct_variant("Error", &self.r)?;

                        sv.end();

                        unimplemented!()
                    }
                    _ => unimplemented!()
                }
            }
            _ => unimplemented!()
        }
        
        // state.serialize_field("r", &self.r)?;
        // state.serialize_field("g", &self.g)?;
        // state.serialize_field("b", &self.b)?;
        
        //state.end()
    }
}


impl std::error::Error for Error {}