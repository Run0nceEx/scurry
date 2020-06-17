

impl Negotiate for LuaTcpStream {}

#[async_trait::async_trait]
impl Connector for LuaTcpStream {
    async fn init_connect(addr: SocketAddr) -> Result<Self, Box<dyn std::error::Error>> {
        unimplemented!()
    }
}

impl Negotiate for Box<LuaTcpStream> {}


struct LuaScript {
    name: String,
    fp: std::path::Path,
    

}


#[async_trait::async_trait]
impl Identifier<LuaTcpStream> for Socks5 {
    async fn detect(&self, con: &mut LuaTcpStream) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(true)
    }
}


