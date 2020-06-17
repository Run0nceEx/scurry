

#[derive(Debug)]
pub enum Error {
    NoProtocolFunc,
    #[allow(non_camel_case_types)]
    io(std::io::Error),
    LuaError(mlua::Error),
    Custom(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            _ =>  write!(f, "unimpl")?
        };
        Ok(())
    }
}

impl std::error::Error for Error {}

impl From<Error> for mlua::Error {
    fn from(e: Error) -> mlua::Error {
        mlua::Error::external(e)
    }
}

impl From<mlua::Error> for Error {
    fn from(e: mlua::Error) -> Error {
        Self::LuaError(e)    
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::io(e)
    }
}

