use std::fs::File;
use std::io::{BufReader, BufRead};

pub enum Boundary {
    Limited(usize),
    Unlimited
}

pub fn get_max_fd() -> Result<Boundary, Box<std::error::Error>> {
    const INDICATOR: &'static str = "Max open files";

    let mut fd = BufReader::new(File::open("/proc/self/limits")?);
    let mut buf = String::new();
    while let Ok(n) = fd.read_line(&mut buf) {
        if buf.trim().starts_with(INDICATOR) {
            if let Some(size) = buf.split("            ").nth(1) {
                if size.starts_with("unlimited") {
                    return Ok(Boundary::Unlimited)
                }
                else {
                    return Ok(Boundary::Limited(size.parse()?))
                }
            }
        }

        if n == 0 {
            break
        }

        buf.clear();
    }

    return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::UnexpectedEof,
        format!("Could not find `{}` in /proc/self/limits", INDICATOR)
    )))
}