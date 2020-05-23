
use mlua::Lua;
use std::collections::HashMap;
use super::vnet::*;
use std::net::SocketAddr;

use tokio::net::TcpStream;

use crate::{Identifier, Connector, Negotiate};

impl Negotiate for VStream {}

#[async_trait::async_trait]
impl Connector for VStream {
    async fn init_connect(addr: SocketAddr) -> Result<VStream, Box<dyn std::error::Error>> {
        Ok(VStream::new(TcpStream::connect(addr).await?))
    }
}

struct Socks5;

#[async_trait::async_trait]
impl Identifier<VStream> for Socks5 {
    async fn detect(&self, con: &mut TcpStream) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}

pub struct Engine {
    interpreter: Lua,
    protocols: Vec<LuaShimCache>
}

pub struct LuaShimCache(String);


impl LuaShimCache {
    fn load<'a>(&self, lua: &'a Lua) -> Result<mlua::Function<'a>, Error> {
        Ok(lua.globals().get(self.0)?)
    }
}

struct LuaConnector;


#[async_trait::async_trait]
impl Identifier<VStream> for LuaShimCache {
    async fn detect(&self, con: &mut VStream) -> Result<bool, Box<dyn std::error::Error>> {   
        Ok(true)
    }
}


impl Engine {
    fn init() -> Result<Self, Error> {
        let engine = Engine {
            interpreter: Lua::new(),
            protocols: Vec::new()
        };

        let globals = &mut engine.interpreter.globals();
        
        let env_var = engine.interpreter.create_table()?;
        globals.set("ENGINE", env_var);
        Ok(engine)
    }

    fn load_cache(&mut self) -> Result<(), Error>{
        let globals = &mut self.interpreter.globals();
        let env_var: mlua::Table = globals.get("ENGINE")?;
        
        self.protocols = env_var.get::<&str, Vec<String>>("ADDONS")?
                            .iter()
                            .map(|x| LuaShimCache(x.to_string()))
                            .collect();
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    NoKeyInPool,
    NoProtocolFunc,

    #[allow(non_camel_case_types)]
    io(std::io::Error),

    LuaError(mlua::Error)
}

impl std::fmt::Display for Error {
    fn fmt(f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!("{:?}");
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

