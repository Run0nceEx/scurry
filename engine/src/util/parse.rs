use tokio::io::BufReader;
use tokio::prelude::*;

use crate::task::meta::CronMeta;
use crate::error::Error;

use std::time::Duration;
use std::net::SocketAddr;

pub async fn read_ip_port_file<T, J>(filepath: T, job_buf: &mut Vec<(CronMeta, J)>) -> Result<(), Error>
where
    T: AsRef<std::path::Path>,
    J: From<SocketAddr>
{
	let file = tokio::fs::File::open(filepath).await?;
	let mut reader = BufReader::new(file);
	let mut buf = String::new();

	while let Ok(n) = reader.read_line(&mut buf).await {
		if buf.len() > 0 {
		    match buf.trim().parse::<std::net::SocketAddr>() {
				Ok(addr) =>  {
					let meta = CronMeta::new(Duration::from_secs_f32(5.0), Duration::from_secs_f32(0.0), 3);
                    
                    let job = J::from(addr);
                    
                    job_buf.push((meta, job));
				},
				
				Err(e) => {
					eprintln!("failed to parse {}", buf.trim())
				}	
			}
		}
		
		if n == 0 {
			break
		}

		buf.clear();

	}
	Ok(())
}
