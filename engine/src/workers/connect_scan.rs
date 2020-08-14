use crate::{
    task::{
        {SignalControl, CRON},
        //pool::Subscriber,
        CronMeta,
    },
    error::Error,
};

use tokio::net::TcpStream;
use std::{net::SocketAddr, time::{Duration, Instant}};

#[derive(Debug, Clone)]
pub struct Job {
    pub addr: SocketAddr
}

impl Job {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr: addr,
        }
    }
}

#[derive(Debug, Clone)]
pub enum PortState {
    Open(SocketAddr),
    Closed(SocketAddr)
}

#[derive(Debug)]
pub struct OpenPortJob;

#[async_trait::async_trait]
impl CRON for OpenPortJob
{
    type State = Job;
    type Response = PortState;

    async fn exec(state: &mut Job) -> Result<(SignalControl, Option<Self::Response>), Error>
    {
        match scan(state.addr).await {
            Ok(_) => Ok(
                (SignalControl::Success(true), Some(PortState::Open(state.addr)))
            ),

            Err(Error::IO(x)) => {
                match x.kind() {
                    std::io::ErrorKind::ConnectionAborted
                    | std::io::ErrorKind::ConnectionReset
                    | std::io::ErrorKind::ConnectionRefused 
                    | std::io::ErrorKind::TimedOut => return Ok((SignalControl::Success(false), Some(PortState::Closed(state.addr)))),

                    std::io::ErrorKind::Other => {
                        if let Some(error_code) = x.raw_os_error() {
                            match error_code {
                                //101         // Network unreachable
                                //| 113       // no route to host
                                //| 92        // failed to bind to interface/protocol
                                //| 
                                //24 =>     // too many file-discriptors open
                                //    return Ok((SignalControl::Retry, None)),
                                
                                _ => {
                                        //tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "Error Code: {}", error_code);
                                        return Ok((SignalControl::Success(false), Some(PortState::Closed(state.addr))))
                                } 
                            };
                        }
                        else {
                            tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "Error Code: [OTHER]");
                            return Ok((SignalControl::Retry, None))
                        }
                    }
                    
                    _ => {
                        tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "unmatched {:#?}", x);
                        return Ok((SignalControl::Retry, None))
                    }
                }                        
            },

            Err(e) => {
                tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "unmatched {:#?}", e);
                return Ok((SignalControl::Retry, None))
            }
        }
    }

    fn name() -> String {
        let x = format!("{:?}", OpenPortJob);
        x
    }
}

async fn scan(addr: SocketAddr) -> Result<(), crate::error::Error> {
    //TODO Add timeout
    TcpStream::connect(addr).await?;
    Ok(())
}
