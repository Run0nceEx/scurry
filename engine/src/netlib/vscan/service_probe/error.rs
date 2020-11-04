
#[derive(Debug)]
enum Error {
    IO(std::io::Error),
    ParsingError {
        line_count: usize,
        //token: Token
    }
}

impl From<std::io::Error> for Error {
    fn from(x: std::io::Error) -> Error {
        Error::IO(x)
    }
}

