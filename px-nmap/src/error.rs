use std::path::PathBuf;

#[derive(Debug)]
pub enum ErrorKind {
    ParseError(String),
    ExpectedToken(FileLocation),
    IO(tokio::io::Error)
}


#[derive(Debug, Clone)]
pub struct FileLocation {
    pub path: PathBuf,
    pub column: usize,
    pub row: usize
}

#[derive(Debug)]
pub struct Error {
    err: ErrorKind,
    file_data: Option<FileLocation>
}

impl Error {
    pub fn new(err: ErrorKind, data: Option<FileLocation>) -> Self {
        Self {
            err,
            file_data: data
        }
    }
}


impl Error {
    #[inline(always)]
    pub fn kind(&self) -> &ErrorKind {        
        &self.err
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}


impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", &self)
    }
}


impl From<tokio::io::Error> for ErrorKind {
    fn from(e: tokio::io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<std::num::ParseIntError> for ErrorKind {
    fn from(x: std::num::ParseIntError) -> Self {
        Self::ParseError(x.to_string())
    }
}

impl From<std::num::ParseFloatError> for ErrorKind {
    fn from(x: std::num::ParseFloatError) -> Self {
        Self::ParseError(x.to_string())
    }
}


impl std::error::Error for Error {}
impl std::error::Error for ErrorKind {}