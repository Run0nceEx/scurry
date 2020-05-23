use tokio::prelude::*;
use super::Scannable;
use tokio::net::TcpStream;
use std::net::SocketAddr;


/// Raw payload `Http 1.0\n\rGET /\n\rHost: example.com`
struct HttpConnectProtocol(String);

const MAX_HEADER_AMOUNT: usize = 64;
const READ_SIZE: usize = 1024;
const GREET: &[u8] = b"HTTP 1.1\n\rGET /\n\r";

struct HttpReq {}

enum HttpError {
    stdio(std::io::Error),
    tokio(tokio::io::Error),
    parse(httparse::Error)
}

impl From<tokio::io::Error> for HttpError {
    fn from(x: tokio::io::Error) -> Self {
        Self::tokio(x)
    }
}

impl From<httparse::Error> for HttpError {
    fn from(x: httparse::Error) -> Self {
        Self::parse(x)
    }
}


#[async_trait::async_trait]
impl Scannable<TcpStream, HttpError> for HttpReq {
    
    async fn connect(&self, addr: SocketAddr) -> Result<TcpStream, HttpError> {
        Ok(TcpStream::connect(addr).await?)
    }

    async fn scan(&self, stream: &mut TcpStream) -> Result<bool, HttpError> {

        stream.write(GREET).await?;

        let mut complete_buf = Vec::with_capacity(READ_SIZE*8);
        stream.read(&mut complete_buf).await?;
        
        {
            let mut resp_buf = Vec::with_capacity(READ_SIZE);
            while stream.read(&mut resp_buf).await? <= 0 {
                complete_buf.extend(resp_buf.iter());
            }
        }

        let mut headers = [httparse::EMPTY_HEADER; MAX_HEADER_AMOUNT];
        let mut req = httparse::Request::new(&mut headers);
        
        if req.parse(&complete_buf[..])?.is_partial() {
            // Todo(Adam): Handle this
            return Ok(false);
        }

        for x in req.headers {
            let mut name = String::from(x.name);
            name.make_ascii_lowercase();
            if name == String::from("status") && x.value == b"200" {
                return Ok(true)
            }
        }

        Ok(false)
    }
}