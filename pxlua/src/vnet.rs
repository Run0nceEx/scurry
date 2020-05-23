use mlua::{UserData, UserDataMethods, Lua, Function};
use tokio::{
    net::TcpStream,
    io::AsyncReadExt,
    prelude::*
};
use std::{
    cell::RefCell,
    io::Read,
    net::SocketAddr
};

use corelib::{Identifier, Connector, Negotiate};

use std::net::Shutdown;
use std::sync::{Arc, Mutex};

use bstr::BString;

#[derive(Clone)]
pub struct LuaTcpStream(Arc<Mutex<RefCell<TcpStream>>>);

impl LuaTcpStream {
    pub fn new(con: TcpStream) -> Self {
        Self(Arc::new(RefCell::new(con)))
    }
}

impl UserData for LuaTcpStream {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("peer_addr", |_, stream, ()| async move {
            Ok(stream.0.borrow().peer_addr()?.to_string())
        });

        methods.add_async_method("read", |_, stream, size: usize| async move {
            let mut buf = vec![0; size];
            let n = stream.0.borrow_mut().read(&mut buf).await?;
            buf.truncate(n);
            Ok(BString::from(buf))
        });

        methods.add_async_method("write", |_, stream, data: BString| async move {
            let n = stream.0.borrow_mut().write(&data).await?;
            Ok(n)
        });

        methods.add_method("close", |_, stream, ()| {
            stream.0.borrow().shutdown(Shutdown::Both)?;
            Ok(())
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::prelude::*;
    use super::super::engine::Error;

    #[test]
    /// Tests if tcp is found in lua, and if the function shim can be ran in rust
    fn tcp_shim_access_single_thread() -> Result<(), Error> {
        tokio::task::spawn_local(async move {
            let lua = Lua::new();
            
            let server = lua.load(
                r#"
                local stream = ...
                local addr = stream:peer_addr()
                print("connected "..addr)
                stream:write("hello")
                
                local data = stream:read(100)
                data = data:match("^%s*(.-)%s*$") -- trim
                print("["..peer_addr.."] "..data)
                stream:close()
                return data == "test"
                "#,
            ).into_function().unwrap();

            assert!(server.call_async::<_, bool>(
                LuaTcpStream::new(TcpStream::connect("1.1.1.1:53").await.unwrap())
            ).await.unwrap());
        });
        Ok(())
    }


    #[test]
    /// Tests if tcp is found in lua, and if the function shim can be ran in rust
    fn tcp_shim_access_multi_thread() -> Result<(), Error> {
        tokio::task::spawn(async move {
            let lua = Lua::new();
            
            let server = lua.load(
                r#"
                local stream = ...
                local addr = stream:peer_addr()
                print("connected "..addr)
                stream:write("hello")
                
                local data = stream:read(100)
                data = data:match("^%s*(.-)%s*$") -- trim
                print("["..peer_addr.."] "..data)
                stream:close()
                return data == "test"
                "#,
            ).into_function().unwrap();

            assert!(server.call_async::<_, bool>(
                LuaTcpStream::new(TcpStream::connect("1.1.1.1:53").await.unwrap())
            ).await.unwrap());
        });
        Ok(())
    }
}