use regex::{Regex, Match, Matches};
use crate::error::Error;

lazy_static::lazy_static! {
    static ref HEX_BYTE: Regex = Regex::new(r"(\\[x][a-hA-H0-9][a-hA-H0-9])*").unwrap();
}

/// When this function is executed, it will look for the pattern 
/// `\xHH` where 'H' is A-F or 0-9 representing a hexidecimal. 
/// For each instance of this pattern (eg. '\x10', '\xFF') inside of `source`, 
/// will be converted from the string representation into a byte.
/// Every other character that doesn't fit the pattern, 
/// it will be converted into utf-8 decimal value in the order received.
/// ```notest
/// let mut buf = Vec::new();
/// construct_payload(r"a\xFFb", &mut buf);
/// assert_eq!(&buf[..], &[61, 255, 62])
/// ```
pub fn construct_payload(source: &str, output: &mut Vec<u8>) -> Result<(), Error> {
    let replacements: Vec<((usize, usize), u8)> = HEX_BYTE.find_iter(source)
        // \x00\xFF -> [0, 255]
        .filter(|m| m.as_str().len() == 4)
        .map(|m| 
            ((m.start(), m.end()), u8::from_str_radix(&m.as_str()[2..], 16).unwrap())
        )   
        .collect();

    let mut last_end = 0;
    for ((start, end), byte) in replacements {
        output.extend(source[last_end..start].as_bytes());
        output.push(byte);
        last_end = end;
    }
    output.extend(source[last_end..].as_bytes());
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_byte_string() {
        let mut buffer = Vec::with_capacity(64);

        let mut f = |s: &str, o: &[u8]| {
            println!("DATA: '{}'", &s);
            construct_payload(s, &mut buffer).unwrap();
            assert_eq!(&buffer[..], o);
            buffer.clear();
        };

        (f)(r"a", &[0x61]);
        (f)(r"a\x01", &[0x61,0x01]);
        (f)(r"a\x01b\xFF", &[0x61, 0x01, 0x62, 0xFF]);
        (f)(r"a\x01b\xFFc", &[0x61, 0x01, 0x62, 0xFF, 0x63]);
    }
}