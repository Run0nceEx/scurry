use async_std::net::TcpStream;
use async_std::io;


pub async fn socks5(s: TcpStream) -> io::Result<()> {Ok(())}

pub async fn http(s: TcpStream) -> io::Result<()> {Ok(())}

pub async fn https(s: TcpStream) -> io::Result<()> {Ok(())}

pub async fn noop(s: TcpStream) -> io::Result<()> {Ok(())}
