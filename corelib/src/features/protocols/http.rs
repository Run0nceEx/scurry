use tokio::prelude::*;
use crate::Identifier;
use tokio::net::TcpStream;

const MAX_HEADER_AMOUNT: usize = 64;
const READ_SIZE: usize = 1024;
const GREET: &[u8] = b"HTTP 1.1\n\rGET /\n\r";

struct HttpReq {}

#[async_trait::async_trait]
impl Identifier<TcpStream> for HttpReq {
    async fn detect(&self, stream: &mut TcpStream) -> Result<bool, Box<std::error::Error>> {

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