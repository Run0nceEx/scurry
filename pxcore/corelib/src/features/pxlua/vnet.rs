
use mlua::{UserData, UserDataMethods};
use tokio::{
    net::TcpStream,
    io::AsyncReadExt,
    prelude::*
};
use std::{
    cell::RefCell,
    io::Read,
    sync::Arc,
};

use slab::Slab;

use super::engine::Error;

const VPOOL_ALLOC: usize = 4098;

lazy_static!{ 
    static ref VPOOL: &'static mut Slab<RefCell<Arc<TcpStream>>> = &mut Slab::with_capacity(VPOOL_ALLOC);
}

#[derive(Clone, Copy)]
pub struct VStream(usize);

impl VStream {
    // technically this is a write to a buffer

    async fn read(&self, buf: &mut [u8]) -> Result<usize, Error> 
    {
        if let Some(mut stream) = VPOOL.get_mut(self.0) {
            let n = stream.into_inner().read(buf).await?;
            return Ok(n)
        }
        Err(Error::NoKeyInPool)
    }

    async fn write(&self, buf: &[u8]) -> Result<(), Error> 
    {
        if let Some(mut stream) = VPOOL.get_mut(self.0) {
            stream.into_inner().write_all(buf).await?;
            return Ok(())
        }
        Err(Error::NoKeyInPool)
    }

    pub fn new(con: TcpStream) -> Self {
        Self(VPOOL.insert(RefCell::new(Arc::new(con))))
    }
}

impl UserData for VStream {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("read", |lua, vstream, nbytes| async move {
            let mut big_buf: Vec<u8> = Vec::with_capacity(nbytes + 1);
            let mut ctr: usize = 0;
            let mut buf: [u8; 1024] = [0; 1024];
            
            loop {
                let n = vstream.read(&mut buf[..]).await?;
                big_buf.extend(&buf[..]);
                
                ctr += buf.len();

                if 0 >= n || ctr >= nbytes {
                    break
                }
                
                buf = [0; 1024];
            }

            Ok(big_buf)
        });


        methods.add_async_method("write", |lua, vstream, buf: Vec<u8> | async move {
            vstream.write(&buf[..]).await?;
            Ok(())
        });
    }
}
