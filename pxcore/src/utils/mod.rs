use serde::{Serialize, Deserialize};

/// raw byte Representation 
pub trait DiskRepr {
    fn save(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> 
    where Self: Serialize + Sized {
        Ok(bincode::serialize(self)?)
    }

    fn load<'a>(buf: &'a [u8]) -> Result<Self, Box<bincode::ErrorKind>> 
    where Self: Deserialize<'a> + Sized {
        Ok(bincode::deserialize(buf)?)
    }

    fn partial_save<T>(self) -> T
    where Self: IntoDiskRepr<T> + Sized, T: Serialize {
        self.into_raw_repr()
    }

    fn partial_load<'a, T>(&mut self, buf: &'a [u8]) -> Result<(), Box<bincode::ErrorKind>>
    where Self: FromDiskRepr<'a, T>, T: Deserialize<'a> {
        self.from_raw_repr(buf)
    }
}



/// Partially serialize into bytes
pub trait IntoDiskRepr<T> {
    fn into_raw_repr(self) -> T where T: Serialize;
}

/// Partially Deserialization from bytes into original
pub trait FromDiskRepr<'a, T> {
    fn from_raw_repr(&mut self, buf: &'a [u8]) -> Result<(), Box<bincode::ErrorKind>>
    where T: Deserialize<'a>;
}
