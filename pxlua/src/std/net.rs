use mlua::{UserData, UserDataMethods};
use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
    prelude::*
};
use std::net::Shutdown;
use std::sync::{Arc, Mutex};
use bstr::BString;


use crate::error::Error;

pub struct Connection;


impl UserData for Connection {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {

    }
}


#[derive(Clone)]
pub struct LuaTcpStream(Arc<Mutex<TcpStream>>);

impl<'a> LuaTcpStream {
    pub fn new(con: TcpStream) -> Self {
        Self(Arc::new(Mutex::new(con)))
    }
}

impl UserData for LuaTcpStream {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("peer_addr", |_, vstream, ()| async move {
            let stream = vstream.0.lock().unwrap();
            Ok(stream.peer_addr()?.to_string())
            
        });

    
        methods.add_async_method("read_all", |_, vstream, () | async move {
            let mut buf = vec![0; 4086];
            let mut stream = vstream.0.lock().unwrap();
            
            let mut n = 1;
            while n > 0 {
                n = stream.read(&mut buf).await?;
            }

            Ok(BString::from(buf))
            
        });

        methods.add_async_method("read_exact", |_, vstream, size: usize| async move {
            let mut buf = vec![0; size];
            let mut stream = vstream.0.lock().unwrap();
            stream.read_exact(&mut buf).await?;
            Ok(BString::from(buf))
        });
        
        methods.add_async_method("write", |_, stream, data: BString| async move {
            let mut rstream = stream.0.lock().unwrap();
            let n = rstream.write(&data).await?;
            
            if n == 0 {
              return Err(mlua::Error::external("memory buffer didn't write"))
            }

            return Ok(())
        });
        

        methods.add_method("close", |_, stream, ()| {
            let mut stream = stream.0.lock().unwrap();
            stream.shutdown(Shutdown::Both);
            Ok(())
        });
    }
}
