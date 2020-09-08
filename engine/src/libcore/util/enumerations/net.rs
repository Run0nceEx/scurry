

use tokio::net::TcpStream;


const HOSTLIST: &'static [&'static str] = &[
    // Quad9
    "9.9.9.9",

    // OpenDNS
    "208.67.222.222",
    "208.67.220.220",

    // googleDNS
    "8.8.8.8",
    "8.8.4.4",

    // cloudflare DNS
    "1.1.1.1",
    "1.0.0.1",

    //Freenon
    "80.80.80.80",
    "80.80.81.81",

];

/// Checks if we're connected by some hosts
pub async fn www_available() -> bool {
    for host in HOSTLIST {

        
        match TcpStream::connect(format!("{}:53", host)).await {
            Ok(_) => return true,
            Err(e) => eprintln!("{}", e)
        }
    }

    return false
}
