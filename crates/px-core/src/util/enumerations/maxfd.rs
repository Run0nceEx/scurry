use std::fs::File;
use std::io::{BufReader, BufRead};

#[derive(Debug, Copy, Clone)]
pub enum Boundary {
    Limited(usize),
    Unlimited
}

impl std::fmt::Display for Boundary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Boundary::Limited(x) => write!(f, "Limited({})", x),
            Boundary::Unlimited => write!(f, "Unlimited")
        }
    }
}


#[cfg(target_os = "linux")]
pub fn get_max_fd() -> Result<Boundary, Box<dyn std::error::Error>> {
    // to raise limits
    // sysctl -w fs.file-max=100000
    // OR
    // vi /etc/sysctl.conf
    // fs.file-max = 100000
    // /proc/sys/fs/file-max - system max
    // /proc/sys/fs/file-nr  - in use
    // ulimit can alleive tension

    const INDICATOR: &'static str = "Max open files";

    let mut fd = BufReader::new(File::open("/proc/self/limits")?);
    let mut buf = String::new();
    
    while let Ok(n) = fd.read_line(&mut buf) {
        if n == 0 {
            break
        }
        
        else if buf.trim().starts_with(INDICATOR) {
            let delimiter = "            ";

            if let Some(size) = buf.split(&delimiter).nth(1) {
                if size.starts_with("unlimited") {
                    return Ok(Boundary::Unlimited)
                }
                else {
                    return Ok(Boundary::Limited(size.parse()?))
                }
            }
        }
        
        buf.clear();
    }

    return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        format!("Could not find `{}` in /proc/self/limits", INDICATOR)
    )))
}


//TODO: figure out how to calculate the maximum amount of a file descriptors a process can own.
#[cfg(target = "windows")]
pub fn get_max_fd() -> Result<Boundary, Box<dyn std::error::Error>> {
    todo!()
}