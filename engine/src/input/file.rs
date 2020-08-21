use std::fs::File;
use std::io::{BufReader, BufRead};

use super::parser::{address_parser, AddressInput};
use crate::error::Error;

pub struct InputFile {
	reader: BufReader<File>,
	line: String
}

impl InputFile {
	pub fn open<T>(path: T) -> Result<Self, Error> where T: AsRef<std::path::Path> {
		Ok(Self {	
			reader: BufReader::new(File::open(path)?),
			line: String::new(),			
		})
	}
}

impl Iterator for InputFile {
	type Item = AddressInput;

	fn next(&mut self) -> Option<Self::Item> {
		let mut line_count = 0;
		
		loop {
			if let Ok(nbytes) = self.reader.read_line(&mut self.line) {
				line_count += 1;
				if nbytes == 0 {
					return None
				}

				match address_parser(self.line.as_str()) {
					Ok(addr) => {
						if let AddressInput::File(_) = addr {
							eprintln!("File parsing error on line {}", line_count);
							continue
						}

						return Some(addr)
					},
					
					Err(e) => { 
						eprintln!("File parsing error on line {} [{}]", line_count, e); 
						continue
					}
				};

				
			}

			else {
				return None
			}
		
		}
	}
}

