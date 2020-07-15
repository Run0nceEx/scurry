use std::collections::HashMap;

use mlua::{Lua, UserData};

use super::error::Error;
use super::{
    std::net::{handler::Handler}

};
use corelib::{ConnectionHandler};


fn init<T>(handlers: HashMap<String, Handler<T>>) -> Result<Lua, Error>
where 
    T: ConnectionHandler<T> + Clone + 'static + Sync + Send + UserData
{
    let mut engine = Lua::new();
    
    engine.globals().set("HANDLERS", handlers)?;

    engine.globals().set("print", 
        engine.create_function(|_lua, x: String| { println!("{}", x) })?
    )?;

    engine.globals().set("eprint", 
        engine.create_function(|_lua, x: String| { eprintln!("{}", x) })?
    )?;
    
    engine.globals().set("spawn", 
        engine.create_function(move |_, func: mlua::Function| {
            tokio::task::spawn_local(async move { func.call_async::<_, ()>(()).await.unwrap() });
            Ok(())
        })?
    )?;
    
    
    Ok(engine)
}




// impl<T> IntoHandler<T> for T 
// where 
//     T: ConnectionHandler<LuaTcpStream> + 'static + Into<LuaTcpStream>, 
// {}