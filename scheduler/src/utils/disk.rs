use serde::{Serialize, Deserialize};

pub enum Mode {
    Bin,
    Json,
}

impl Mode {
    fn deserialize<'a, T>(&self, buf: &'a [u8]) -> Result<T, Box<dyn std::error::Error>>
    where T: Deserialize<'a> {
        match self {
            Self::Bin => Ok(bincode::deserialize::<T>(buf)?),
            _ => unimplemented!()
        }
    }

    fn serialize<T>(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> where T: Serialize {
        match self {
            Self::Bin => Ok(bincode::serialize(data)?),
            _ => unimplemented!()
        }
    }
}

/// raw byte Representation 
pub trait DiskRepr {
    /// Serialize `Self` into raw bytes
    #[inline]
    fn save(&self, output: Mode) -> Result<Vec<u8>, Box<dyn std::error::Error>> 
    where Self: Serialize + Sized {
        output.serialize(self)
    }

    /// deserialize raw bytes into `Self` 
    #[inline]
    fn load<'a>(buf: &'a [u8], input: Mode) -> Result<Self, Box<dyn std::error::Error>> 
    where Self: Deserialize<'a> + Sized {
        input.deserialize(buf)
    }

    /// Partially serialize `self` into bytes
    #[inline]
    fn partial_save<T>(self) -> T
    where Self: IntoDiskRepr<T> + Sized, T: Serialize {
        self.into_raw_repr()
    }

    /// Partially deserialize into `self`
    #[inline]
    fn partial_load<'a, T>(&mut self, buf: &'a [u8], input: Mode) -> Result<(), Box<dyn std::error::Error>>
    where Self: FromDiskRepr<'a, T>, T: Deserialize<'a> {
        self.from_raw_repr(buf, input)
    }
}

/// Partially serialize into bytes
pub trait IntoDiskRepr<T> {
    fn into_raw_repr(self) -> T where T: Serialize;
}

/// Partially Deserialization from bytes into original
pub trait FromDiskRepr<'a, T> {
    fn from_raw_repr(&mut self, buf: &'a [u8], input: Mode) -> Result<(), Box<dyn std::error::Error>>
    where T: Deserialize<'a>;
}

