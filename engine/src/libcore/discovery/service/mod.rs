// contains code for discovering what service is running behind a port

mod nmap;
pub mod connect_scan;
pub mod socks5;

use crate::libcore::task::SignalControl;
use std::time::Duration;

fn handle_io_error<T>(x: std::io::Error, refused_value: T) -> (SignalControl, Option<T>) {
    match x.kind() {
        std::io::ErrorKind::ConnectionAborted
        | std::io::ErrorKind::ConnectionReset
        | std::io::ErrorKind::ConnectionRefused 
        | std::io::ErrorKind::TimedOut
        => (SignalControl::Success(false), Some(refused_value)),

        std::io::ErrorKind::Other => {
            if let Some(error_code) = x.raw_os_error() {
                match error_code {
                    101         // Network unreachable
                    | 113       // no route to host
                    | 92        // failed to bind to interface/protocol
                    | 24 =>     // too many file-discriptors open
                        return (SignalControl::Stash(Duration::from_secs(5)), None),
                    
                    _ => {
                            tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "Error Code: {}", error_code);
                            return (SignalControl::Success(false), Some(refused_value))
                    } 
                };
            }
            else {
                tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "Error Code: [OTHER]");
                return (SignalControl::Retry, None)
            }
        }
        
        
        _ => {
            tracing::event!(target: "Schedule Thread", tracing::Level::WARN, "unmatched {:#?}", x);
            return (SignalControl::Retry, None)
        }
    
    }  
}