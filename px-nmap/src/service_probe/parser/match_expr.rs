use std::{
    str::FromStr,
    time::Duration
};

use px_core::model::PortInput;

use crate::error::Error;
use super::{
    cpe::{Identifier, CPExpr}, 
    model::{Token, Directive, DataField}
};

use logos::Lexer;
use smallvec::SmallVec;

#[derive(Debug, PartialEq, Eq)]
pub enum Flags {
    UNIT,
    CaseSensitive,
    IgnoreWhiteSpace    
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
    pub pattern: String,
    pub flags: [Flags; 2],
    pub match_type: Directive,
    pub name: String,

    //TODO(adam)
    pub service_info: ServiceInfoExpr,
    pub cpe: SmallVec<[CPExpr; 8]>
}


pub fn parse_match_expr(line_buf: &str, lex: &mut Lexer<Token>) -> Result<MatchLineExpr, Error> {
    // parsing example
    // match insteon-plm m|^\x02\x60...(.).\x9b\x06$| p/Insteon SmartLinc PLM/ i/device type: $I(1,">")/
    // -----
    // match insteon-plm ...
    // ^ we're here
    let mut tokens = line_buf.split_whitespace();
    let directive = Directive::from_str(tokens.next().ok_or_else(|| Error::ExpectedToken(Token::Match))?)?;
    
    let mut service_data = ServiceInfoExpr::new();
    let mut pattern: &str;

    // setup buffer for saving flags
    let mut flags: SmallVec<[Flags ; 2]> = SmallVec::new();
        
    let name = tokens.next()
        .ok_or_else(|| Error::ExpectedToken(Token::Word))?;
    // match insteon-plm ...
    //       ^ we're here

    // match insteon-plm m|^\x02\x60...
    //                   ^ we're here
    let partial_query = tokens.next()
        .ok_or_else(|| Error::ParseError(format!("Expected Regex pattern to occur")))?;
    
    // now instead of spliting by spaces, we will just iter over the characters partially

    let mut cursor = partial_query.chars();
    // m|^\x02\x60...
    // ^ we're here
    let regex_char = cursor.next().ok_or_else(|| Error::ParseError(String::from("Expected delimiter")))?;
    if regex_char == 'm' {
        // m|^\x02\x60....
        //  ^ we're here
        let delimiter = cursor.next()
            .ok_or_else(|| Error::ParseError(String::from("Expected delimiter")))?;
        
        
        // now we split on '|' (delimiter)
        let mut regex_cursor = line_buf.split(delimiter);
        
        // everything before '|'
        // 'match insteon-plm m'
        let head = regex_cursor.next()
            .ok_or_else(|| Error::ParseError(String::from("No match on delimiter")))?;
        
        // regex pattern
        // '^\x02\x60...(.).\x9b\x06$'
        pattern = regex_cursor.next() // grabs the pattern
            .ok_or_else(|| Error::ParseError(String::from("No match on delimiter")))?;
        
        
        // everything after the second '|'
        let tail = regex_cursor.next()
            .ok_or_else(|| Error::ParseError(String::from("No match on delimiter")))?;
        
        // offset to enumerate where the common platform enumeration is
        let mut offset = 0;
        
        // sanity check
        if head.len() >= 1 && pattern.len() >= 1 {
            let head = head.len()-1;
            let pattern_len = pattern.len()-1;

            // +2 for regex delimiters
            offset = head+pattern_len+2; 
        }

        else {
            return Err(Error::ParseError(format!("error reading line: {}", line_buf)))
        }
        
        for c in tail.chars() {
            offset += 1;
            if c == ' ' {
                break 
            }
            let flag = match c {
                'i' => Flags::IgnoreWhiteSpace,
                's' => Flags::CaseSensitive,
                flag => return Err(
                    Error::ParseError(format!("unknown flag ({}) found in: {}", flag, line_buf))
                )
            };

            if !flags.contains(&flag) {
                flags.push(flag)
            }
        }
        
        //  p/Android Debug Bridge/
        //  i/auth required: $I(1,"<")/
        //  o/Android/
        //  cpe:/o:google:android/a
        //  cpe:/o:linux:linux_kernel/a
        let mut cursor = line_buf[offset..].chars();
        let mut field_buf: String = String::with_capacity(256);

        let mut ignore_space = true;
        let mut inside_field = false;
        let mut delimiter: char = '/';
        let mut service: Option<char> = None; 

        const WHITESPACE: [char; 4] = [' ', '\n', '\t', '\r'];
        const SERVICE_INDENTS: [char; 5] = ['p', 'v', 'i', 'o','d'];

        while let Some(c) = cursor.next() {
            if ignore_space {
                if SERVICE_INDENTS.contains(&c) {
                    delimiter = cursor.next().unwrap();
                    ignore_space = false;
                    inside_field = true;
                    service = Some(c);
                }
                else if WHITESPACE.contains(&c) { continue }
                else if c == 'c' { // cpe:/ ?
                    let mut word: [char; 5] = [' '; 5];
                    word[0] = 'c';

                    for i in 1..4 {
                        word[i] = cursor.next().unwrap();
                    }
                    
                    if word == ['c', 'p', 'e', ':', '/'] {
                        
                    }
                    else {
                        // que ? bad input ? no computo hambre
                    }
                }

                else { 
                    // just ensure our flags are setup correctly
                    // super cryptic - ignore_space == false && inside_field == true)
                    if crappy_xor(ignore_space, inside_field) {
                        // we're at the first delimiter
                        while let Some(c) = cursor.next() {
                            if c != delimiter {
                                field_buf.push(c)
                            }
                            else { break }
                        }
                        // follow until delimiter to collect field
                        match service.unwrap() {
                            'p' => buf_clone(&mut service_data.product_name, &field_buf),
                            'v' => buf_clone(&mut service_data.version, &field_buf),
                            'i' => buf_clone(&mut service_data.info, &field_buf),
                            'o' => buf_clone(&mut service_data.operating_system, &field_buf),
                            'd' => buf_clone(&mut service_data.device_type, &field_buf),
                            'h' => buf_clone(&mut service_data.hostname, &field_buf),
                            c => return Err(Error::ParseError(format!("expected charact in {:?}, got '{}'", SERVICE_INDENTS, c)))
                        }
                        service = None;
                        field_buf.clear();
                    }
                    else {
                        return Err(
                            Error::ParseError(format!(
                              "expected version information or CPE identifier, instead got '{}'", c))
                        )
                    }
                }
            }
        }
        
        
        // match Identifier::from_char(service_data.chars().nth(0).unwrap())? {
        //     _ => {}
        // }

    }

    else {
        // syntax error?
        // match <name> m<pattern> [<version> ...]
        return Err(
            Error::ParseError(format!(
                "unknown sequence expected 'm', instead got '{}' inside of '{}' ",
                regex_char, line_buf
            ))
            
        );
    }

    // Ok(MatchLineExpr {
    //     name,
    //     pattern,
    //     flags: flags[..],
    //     match_type: Directive::from_str(first_token)?,  
    // })

    unimplemented!()
}

fn buf_clone(buf: &mut Option<DataField>, field: &str) {
    *buf = Some(field.clone().into());
}

#[inline(always)]
fn crappy_xor(a: bool, b: bool) -> bool { a != b || a || b }