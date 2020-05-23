
use mlua::Lua;
use std::net::SocketAddr;

use mlua::{UserData, UserDataMethods};
use tokio::{
    net::TcpStream,
    io::AsyncReadExt,
    prelude::*
};
use std::{
    cell::RefCell,
    io::Read
};


use super::vnet::LuaTcpStream;
use corelib::{Negotiate, Identifier, Connector};


impl Negotiate for LuaTcpStream {}

#[async_trait::async_trait]
impl Connector for LuaTcpStream {
    async fn init_connect(addr: SocketAddr) -> Result<LuaTcpStream, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}



struct Socks5;

#[async_trait::async_trait]
impl Identifier<LuaTcpStream> for Socks5 {
    async fn detect(&self, con: &mut LuaTcpStream) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}

impl Negotiate for Box<LuaTcpStream> {}

#[async_trait::async_trait]
impl<'a> Identifier<Box<LuaTcpStream>> for mlua::Function<'a> {
    async fn detect(&self, con: &mut Box<LuaTcpStream>) -> Result<bool, Box<dyn std::error::Error>> {   
        unimplemented!()
    }
}


// impl Engine {
//     fn init() -> Result<Self, Error> {
//         let engine = Engine {
//             interpreter: Lua::new(),
//             protocols: Vec::new(),
//             vpool: VPool::new()
//         };

//         let globals = &mut engine.interpreter.globals();
        
//         let env_var = engine.interpreter.create_table()?;
//         globals.set("ENGINE", env_var);
//         Ok(engine)
//     }

//     fn load_cache(&mut self) -> Result<(), Error>{
//         let globals = &mut self.interpreter.globals();
//         let env_var: mlua::Table = globals.get("ENGINE")?;
        
//         self.protocols = env_var.get::<&str, Vec<String>>("ADDONS")?
//                             .iter()
//                             .map(|x| LuaShimCache(x.to_string()))
//                             .collect();
//         Ok(())
//     }
// }

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

