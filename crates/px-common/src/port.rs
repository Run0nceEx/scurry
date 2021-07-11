#[derive(Debug)]
pub enum Error {
    BadRange,
    BadInt,
    IO(std::io::Error)
}

impl From<std::num::ParseIntError> for Error {
    fn from(x: std::num::ParseIntError) -> Self {
        Self::BadInt
    }
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Self::IO(x)
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortInput {
    Singleton(u16),
    Range(std::ops::Range<u16>)
}

impl PortInput {
    pub fn contains(&self, other: u16) -> bool {
        match self {
            PortInput::Singleton(s) => s.eq(&other),
            PortInput::Range(rng) => rng.contains(&other)
        }
    }
}

impl std::str::FromStr for PortInput {
    type Err = Error;

    /// Accepts '80-443', '80', '0-10'
    fn from_str(src: &str) -> Result<Self, Self::Err> {
        port_parser(src)
    }
}

pub fn port_parser(src: &str) -> Result<PortInput, Error> {
    let data: Vec<&str> = src.split("-").collect();
    let mut bottom = data.get(0).unwrap().parse::<u16>()?;

    if data.len() > 2 {
        //std::num::ParseIntError::
        return Err(Error::BadInt)
    }
    
    if data.len() == 2 {
        let mut top = data.get(1).unwrap().parse::<u16>()?;
        if bottom > top {
            std::mem::swap(&mut bottom, &mut top);
        }
        return Ok(PortInput::Range(bottom..top))
    }

    Ok(PortInput::Singleton(bottom))
}
