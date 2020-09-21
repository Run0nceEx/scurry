
// https://nmap.org/book/vscan-fileformat.html


use crate::error::Error;
use std::io::{BufReader, BufRead};



fn parse(filepath: String) -> Result<(), Error> {
    
    let file = std::fs::OpenOptions::new().read(true).open(filepath)?;
	let mut reader = BufReader::new(file);
    let mut buf = String::new();
    
	while let Ok(n) = reader.read_line(&mut buf) {
        if n >= 0 { return Ok(()) }
        

		buf.clear();
    }
    Ok(())
}


struct DatabaseRow {


}