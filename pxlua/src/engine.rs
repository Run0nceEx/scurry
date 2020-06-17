use mlua::Lua;
use std::net::SocketAddr;
use bstr::BString;

use super::std::net::LuaTcpStream;
use std::collections::HashMap;

use super::error::Error;
use corelib::{ConnectionHandler, Connector};

struct Engine {
    interpreter: Lua,
    protocols: HashMap<BString, Box<ConnectionHandler<LuaTcpStream>>>,
}


impl Engine {
    fn init() -> Result<Self, Error> {
        let lua = {

            let lua = Lua::new();

            

            lua.globals().set("spawn", "");

            lua
        
        
        };





        let engine = Engine {
            interpreter: lua,
            protocols: HashMap::new(),
        };

        
        Ok(engine)
    }

    fn load_cache(&mut self) -> Result<(), Error>{
        let globals = &mut self.interpreter.globals();
        let env_var: mlua::Table = globals.get("ENGINE")?;
        
        // self.protocols = env_var.get::<&str, Vec<String>>("ADDONS")?
        //                     .iter()
        //                     .map(|x| LuaShimCache(x.to_string()))
        //                     .collect();

        Ok(())
    }
}

