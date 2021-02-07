struct Span {
    start: usize,
    end: usize
}

#[derive(Debug)]
pub enum Error {
    TypeParse(String),
    
    // General syntax reading error
    SyntaxError,

    ExpectedToken,
    UnknownToken(String),


    // already declared, and over riden
    OverLapExpr,

    IO(std::io::Error),    
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Self {
        Self::IO(x)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(x: std::num::ParseIntError) -> Self {
        Self::TypeParse(x.to_string())
    }
}
