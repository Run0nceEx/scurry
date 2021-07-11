use std::str::FromStr;
use smallvec::SmallVec;

use crate::error::Error;
use super::{
    model::{
        Token,
        Directive,
        DataField,
        CPExpr
    },
    api::Meta
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Flags {
    UNIT,
    CaseSensitive,
    IgnoreWhiteSpace
}

impl FromStr for Flags {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Flags::from_char(s.chars().nth(0).unwrap())
    }
}


impl Flags {
    fn from_char(s: char) -> Result<Self, Error> {
        Ok(match s {
            'i' => Flags::IgnoreWhiteSpace,
            's' => Flags::CaseSensitive,
            flag => return Err(
                Error::ParseError(format!("Receieved regex flag '{}', expects 'i' or 's'", flag))
            )
        })
    }
}

#[derive(Debug)]
pub struct ServiceInfoExpr {
    pub product_name: Option<DataField>,
    pub version: Option<DataField>,
    pub operating_system: Option<DataField>,
    pub info: Option<DataField>,
    pub hostname: Option<DataField>,
    pub device_type: Option<DataField>
}


impl ServiceInfoExpr {
    pub fn new() -> Self {
        Self {
            product_name: None,
            version: None,
            operating_system: None,
            info: None,
            hostname: None,
            device_type: None
        }
    }
}


#[derive(Debug)]
pub struct MatchLineExpr {
    // with the syntax being m/[regex]/[opts].
    // The “m” tells Nmap that a match string is beginning. 
    // The forward slash (/) is a delimiter,
    // which can be substituted by almost any printable character as long
    // as the second slash is also replaced to match.
    // The regex is a Perl-style regular expression.
    // This is made possible by the
    // excellent Perl Compatible Regular Expressions (PCRE) library
    // (http://www.pcre.org). 
    pub directive: Directive,
    pub name: String,
    pub pattern: RegexExpr,

    pub service_info: ServiceInfoExpr,
    pub cpe: SmallVec<[CPExpr; 8]>
}

pub fn parse_match_expr(line_buf: &str, meta: &mut Meta) -> Result<MatchLineExpr, Error> {
    // parsing example
    // match insteon-plm m|^\x02\x60...(.).\x9b\x06$| p/Insteon SmartLinc PLM/ i/device type: $I(1,">")/
    
    let mut service_data = ServiceInfoExpr::new();
    let mut offset: usize = 0;
    let mut cpe_buf: SmallVec<[CPExpr; 8]> = SmallVec::new();
    
    let mut tokens = line_buf.split_whitespace();
    // -----
    // match insteon-plm ...
    // ^ we're here
    let directive = {
        let data = tokens.next()
          .ok_or_else(|| Error::ExpectedToken(Token::Word))?;
        offset += data.len() + 1;
        Directive::from_str(data)?
    };
    
    // match insteon-plm ...
    //       ^ we're here
    let name = tokens.next()
        .ok_or_else(|| Error::ExpectedToken(Token::Word))?;
    offset += 1 + name.len();

    drop(tokens);

    // match insteon-plm m|^\x02\x60...
    //                   ^ we're here
    let (re_expr, mut cpe_offset) = parse_regex(&line_buf[offset..])?;
    // the rest of
    //  p/Android Debug Bridge/x
    //  i/auth required: $I(1,"<")/
    //  o/Android/
    //  cpe:/o:google:android/a
    //  cpe:/o:linux:linux_kernel/a
    // eprintln!("{}", &line_buf[offset+cpe_offset+1..]);
    
    // eprintln!("{}", &line_buf[offset+cpe_offset+1..]); ??? I tohught +1?
    //eprintln!("{},{},{},{},[{}]", offset, cpe_offset, line_buf.len(), line_buf.len() - (offset+cpe_offset), meta.col);
    
    if line_buf.len() - (offset+cpe_offset) > 0 {
        parse_tail(&line_buf[offset+cpe_offset+1..].trim(), &mut service_data, &mut cpe_buf)?;
    }

    Ok(MatchLineExpr {
        directive,
        name: name.to_string(),
        pattern: re_expr,
        service_info: service_data,
        cpe: cpe_buf
    })
}

#[derive(Debug, Clone)]
pub struct RegexExpr {
    pub delimiter: char,
    pub schematic: String,
    pub flags: [Flags; 2],
}


fn parse_regex(buf: &str) -> Result<(RegexExpr, usize), Error> {
    let mut cursor = buf.chars();
    let regex_char = cursor.next().ok_or_else(|| Error::ParseError(String::from("Expected delimiter")))?;

    if regex_char == 'm' {
        // m|^\x02\x60....
        //  ^ we're here
        let delimiter = cursor.next()
            .ok_or_else(|| Error::ParseError(String::from("Expected delimiter")))?;
        
        // now we split on '|' (delimiter)
        let mut regex_cursor = buf.split(delimiter);
        
        // everything before '|'
        // 'match insteon-plm m'
        let head = regex_cursor.next()
            .ok_or_else(|| Error::ParseError(String::from("No match on delimiter")))?;
        
        // regex pattern
        // '^\x02\x60...(.).\x9b\x06$'
        let pattern = regex_cursor.next() // grabs the pattern
            .ok_or_else(|| Error::ParseError(String::from("No match on delimiter")))?;
        
        // everything after the second '|' (delimiter)
        let tail = regex_cursor.next()
            .ok_or_else(|| Error::ParseError(String::from("No match on delimiter")))?;
        
        // sanity check
        if head.len() >= 1 && pattern.len() >= 1 {
            let head = head.len();
            let pattern_len = pattern.len();

            // offset to seek where to cotinue parsing from
            // +2 for regex delimiters
            let mut offset = head+pattern_len+2;
            
            let mut regex_expr = RegexExpr {
                delimiter,
                schematic: {
                    let mut x = pattern.to_string();
                    x.shrink_to_fit();
                    x
                },
                flags: [Flags::UNIT; 2]
            };

            let mut flag_idx: usize = 0;
            // eprintln!("{}", tail.len());
            for c in tail.char_indices().take_while(|c| c.1 != ' ') {
                let flag = Flags::from_char(c.1)?;
                if !regex_expr.flags.contains(&flag) {
                    regex_expr.flags[flag_idx] = flag;
                    flag_idx += 1
                }
                offset += 1;
            }
            // eprintln!("{}", &buf[..offset]);
            return Ok((regex_expr, offset))
            
        }
        else { return Err(Error::ParseError(format!("error reading line: {}, did not find a head/tail", buf))) }
    }
    else {
        return Err(Error::ParseError(format!("recieved '{}' (expected 'm') in {}", regex_char, buf)))
    }
}

fn parse_field_delimited(data: &str) -> Result<((char, String), Option<&str>), Error> {
    let mut chars = data.chars();

    let head = chars.next()
        .ok_or_else(|| Error::ParseError(String::from(
            "Expected generic character (probably a serviceinfo flag ex 'v/1.0/') but received None"
        )))?;
    
    let delim = chars.next()
        .ok_or_else(|| Error::ParseError(String::from(
            "Expected delimitating character (probably a serviceinfo delimiter ex 'v/1.0/') but received None"
        )))?;

    let schematic: String = chars.take_while(|c| *c != delim).collect();
    
    let mut tail = match &data[schematic.len()+2..].len() {
        0  => None,
        1 => Some(&data[schematic.len()+3..]),
        _ => Some(&data[schematic.len()+4..])
    };
    
    if let Some(buf) = tail {
        if buf.is_empty() {
            tail = None;
        }
    }

    // eprintln!("TAIL({:?}): '{:?}' {}", &data.chars().nth(schematic.len()+2), tail, &data[schematic.len()+2..].len() );

    Ok(((head, schematic), tail))
}

//TODO(adam) bug fix
//  CPE's need the ability to know of the `/a` sequence at the end of the CPE
// **and**, with that, 
fn parse_cpe_expr(data: &str) -> Result<(&str, Option<&str>), Error> {
    if data.starts_with("cpe:") {
        let mut schematic = String::with_capacity(256);

        let mut chars = data.trim().char_indices().skip(4);
        schematic.push_str("cpe:");
        
        let delim = chars.next().ok_or_else(|| Error::ParseError(String::from(
            "Expected delimitating character (probably a serviceinfo delimiter ex 'v/1.0/') but received None"
        )))?.1;

        let mut tail_idx = chars.take_while(|c| c.1 != delim).last().unwrap().0;

        if data[tail_idx+1..].len() > 1 {
            let a_flag = data.chars().nth(tail_idx+2).unwrap();
            if a_flag == 'a' {
                tail_idx += 1;
            }
        }
        
        let tail = match &data[2+tail_idx..].len() {
            0 | 1 => None,
            _ => Some(&data[2+tail_idx..])
        };

        if data.chars().nth(tail_idx+1).unwrap() == delim {
            tail_idx -= 1;
        }
        return Ok((&data[5..tail_idx+2], tail))
    }
    
    else {
        return Err(Error::ParseError(format!(
            "got '{:?}' but expected sequence: 'cpe:/.../'", data  
        )));
    }
}
/// Parses tail end expression of the match statement, exampling the following
///  ```notest
///     p/Android Debug Bridge/
///     i/auth required: $I(1,"<")/
///     o/Android/
///     cpe:/o:google:android/a
///     cpe:/o:linux:linux_kernel/a
/// ```
fn parse_tail(mut buf: &str, expr: &mut ServiceInfoExpr, cpes: &mut SmallVec<[CPExpr; 8]>) -> Result<(), Error> {
    const SERVICE_IDENT: [char; 6] = ['p', 'v', 'i', 'o','d', 'h'];
    while buf.len() > 0 {
        match parse_cpe_expr(&buf) {
            Ok((cpe, tail)) => {
                //eprintln!("{:?}, {:?}", cpe, tail);

                cpes.push(CPExpr::new(DataField::from(cpe.to_string())));
                match tail {
                    Some(tail) => buf = &tail[1..],
                    None => break
                }
            },
            
            Err(Error::ParseError(_)) => {
                // lets just ambigiously check its not a serviceinfo field
                // instead of making a proper solution on how to parse the remaining fields
                // if the CPE parser errors, we try this instead.
                let ((head, schematic), tail) = parse_field_delimited(&buf)?;
                match head {
                    'p' => buf_clone(&mut expr.product_name, &schematic),
                    'v' => buf_clone(&mut expr.version, &schematic),
                    'i' => buf_clone(&mut expr.info, &schematic),
                    'o' => buf_clone(&mut expr.operating_system, &schematic),
                    'd' => buf_clone(&mut expr.device_type, &schematic),
                    'h' => buf_clone(&mut expr.hostname, &schematic),
                    c => return Err(Error::ParseError(format!("expected character in {:?}, got '{}'\n{}", SERVICE_IDENT, c, buf)))
                };

                match tail {
                    Some(x) => buf = x,
                    None => break
                }
            }
            
            Err(e) => return Err(e)
        }
    }
    Ok(())
}



#[inline(always)]
fn buf_clone(buf: &mut Option<DataField>, field: &str) { 
    if field.len() > 0 {
        *buf = Some(field.clone().into()); 
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_regex_seg() {
        const DATA: &'static str = "m/some data/si";
        let (re, offset) = parse_regex(&DATA).unwrap();
        
        assert_eq!(re.flags, [Flags::CaseSensitive, Flags::IgnoreWhiteSpace]);
        assert_eq!(re.schematic, "some data");
        assert_eq!(DATA.len(), offset);
    }

    #[test]
    fn parse_service_info_seg() {
        const DATA: &'static str = "v/some data/";
        let ((head, cpe), tail) = parse_field_delimited(&DATA).unwrap();
        assert_eq!(head, 'v');
        assert_eq!(cpe, "some data");
        assert_eq!(tail, None);
    }

    #[test]
    fn parse_service_info() {
        const SECTIONS: &'static [(char, &str)] = &[('v', "some data"), ('i', "some crazy data"), ('h',"yeep")];
        const DATA: &'static str = "v/some data/ i/some crazy data/ h/yeep/";

        let mut sect_idx = 0;
        let mut data = DATA.clone();

        while let ((head, field), Some(tail)) = parse_field_delimited(&data).unwrap() {
            assert_eq!(field, SECTIONS[sect_idx].1);
            assert_eq!(head, SECTIONS[sect_idx].0);
            sect_idx += 1;
            data = &tail[1..];
        }
    }

    #[test]
    fn parse_cpe_seg() {
        const DATA: &'static str = "cpe:/a:bearware:teamtalk/";
        let (cpe, tail) = parse_cpe_expr(&DATA).unwrap();
        
        assert_eq!(tail, None);
        assert_eq!(cpe, "a:bearware:teamtalk")
    }

    #[test]
    fn parse_cpe() {
        const SECTIONS: &'static [&'static str] = &["a:bearware:teamtalk", "o:google:android", "o:linux:linux_kernel"];
        const DATA: &'static str = "cpe:/a:bearware:teamtalk/ cpe:/o:google:android/ cpe:/o:linux:linux_kernel/";
        
        let mut sect_idx = 0;
        let mut data = DATA.clone();

        while let (cpe, Some(tail)) = parse_cpe_expr(&data).unwrap() {
            assert_eq!(cpe, SECTIONS[sect_idx]);
            sect_idx += 1;
            data = &tail[1..];
        }

    }

    #[test]
    fn parse_match_line() {
        const DATA: &'static str = r"match socks5 m|^\x05\0\x05\0\0\x01.{6}HTTP|s i/No authentication required; connection ok/ cpe:/a:bearware:teamtalk/";
        let mut meta = Meta::new("N/A");
        let match_expr = parse_match_expr(&DATA, &mut meta).unwrap();
        
        assert_eq!(match_expr.directive, Directive::Match);
        assert_eq!(match_expr.name.as_str(), "socks5");
        assert_eq!(match_expr.pattern.schematic, r"^\x05\0\x05\0\0\x01.{6}HTTP");
        assert_eq!(match_expr.pattern.flags, [Flags::CaseSensitive, Flags::UNIT]);
        assert_eq!(match_expr.service_info.info.unwrap().into_inner().as_str(), "No authentication required; connection ok");
        
        let cpe = match_expr.cpe.get(0).unwrap().clone();
        assert_eq!(cpe.into_inner().as_str(), "a:bearware:teamtalk");
    }
}