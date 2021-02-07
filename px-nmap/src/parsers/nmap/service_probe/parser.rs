use crate::{
    model::PortInput,
    netlib::vscan::service_probe::model::{
        ProbeExpr,
        Protocol,
        Directive,
        Flags,
        MatchLineExpr
    }
};

use super::super::Error;

use std::{
    io::{Read, BufReader, BufRead},
    str::FromStr
};

use smallvec::SmallVec;


const DELIMITER: &'static str = "##############################NEXT PROBE##############################";

pub fn parse_txt_db<T: Read>(fd: &mut BufReader<T>, expressions: &mut Vec<ProbeExpr>, intensity: u8) -> Result<(), Error> {
    let mut line_buf = String::new();
    let mut entity = ProbeExpr::default();
    // parse line by line
    while fd.read_line(&mut line_buf)? > 0 {
        // if probe delimiter reached, attempt to make a `ProbeEntry` out of `ProbeExpr`
        if line_buf.len() == 0 {
            continue
        }

        
        if line_buf.contains(&DELIMITER) {
            if entity.name.len() > 0 && entity.payload.len() > 0 {
                expressions.push(entity);
                entity = ProbeExpr::default();
            }
        }
        
        else if line_buf.trim().starts_with("#") {
            continue
        }
        
        // Cut line where comment begins
        else if line_buf.contains("#") {
            let tail_position = line_buf.chars()
                .take_while(|c| *c != '#')
                .enumerate()
                .map(|(i, c)| i)
                .last()
                .unwrap();
            
            
            if tail_position >= 1 { // safety check (under-flow unsigned)
                let slice = &line_buf[..tail_position-1];
                line_buf = slice.to_string();
            }
        }
        
        // fuck me i hate parsing code ()
        // this is how we're tokenizing i guess
        // dont really feel like building a whole lexer+expr tree
        let mut tokens = line_buf.split_whitespace();        
        
        let first_token = tokens.next().ok_or_else(||Error::ExpectedToken)?;
        
        // parses 'Probe'  
        if first_token.eq("Probe") {
            let protocol = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;
            
            let name = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;
            
            let payload = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?;

            if entity.name.len() > 0 || entity.payload.len() > 0 {
                eprintln!("probe set previously and over written {} -> {}", entity.name, name);
            }

            entity.name = name.to_string();
            entity.proto = Protocol::from_str(protocol)?;
            entity.payload = payload.to_string();
        }
        
        // parse out 'rarity 12'
        else if first_token.eq("rarity") {
            entity.rarity = tokens.next()
                .ok_or_else(|| Error::ExpectedToken)?
                .parse()?;
        }

        // parse out 'match insteon-plm m|^\x02\x60...(.).\x9b\x06$| p/Insteon SmartLinc PLM/ i/device type: $I(1,">")/'
        else if first_token.eq("softmatch") | first_token.eq("match") {
            parse_match_expr(&line_buf, &first_token,  &mut tokens)?;
        }

        else if first_token.eq("ports") {
            while let Some(port) = tokens.next() {
                entity.ports.push(
                    PortInput::from_str(port).unwrap()
                );
            }
        }

        else if first_token.eq("sslports") {
            while let Some(port) = tokens.next() {
                entity.tls_ports.push(
                    PortInput::from_str(port).unwrap()
                );
            }
        }


        else if first_token.eq("totalwaitms") {

        }
        else if first_token.eq("fallback") {}


        line_buf.clear();
    }

    Ok(())
}


fn parse_delimiter<'a>(data: &'a str) -> Result<(char, &'a str), Error>
{
    let mut chars = data.chars();

    let flag = chars.nth(0).ok_or(Error::ExpectedToken)?;
    let delimiter = chars.nth(1).ok_or(Error::ExpectedToken)?;
    


    unimplemented!()
}


fn parse_match_expr(line_buf: &str, first_token: &str, tokens: &mut std::str::SplitWhitespace) -> Result<MatchLineExpr, Error> {
    // parsing example
    // match insteon-plm m|^\x02\x60...(.).\x9b\x06$| p/Insteon SmartLinc PLM/ i/device type: $I(1,">")/
    // -----
    // match insteon-plm ...
    // ^ we're here
    let directive = Directive::from_str(first_token)?;

    let name = tokens.next()
        .ok_or_else(|| Error::ExpectedToken)?;
    // match insteon-plm ...
    //       ^ we're here

    // match insteon-plm m|^\x02\x60...
    //                   ^ we're here
    let partial_query = tokens.next()
        .ok_or_else(|| Error::ExpectedToken)?;
    
    // now instead of spliting by spaces, we will just iter over the characters partially

    let mut cursor = partial_query.chars();
    
    // m|^\x02\x60...
    // ^ we're here
    let regex_char = cursor.next().ok_or_else(|| Error::ExpectedToken)?;
    if regex_char == 'm' {
        // m|^\x02\x60....
        //  ^ we're here
        let delimiter = cursor.next()
            .ok_or_else(|| Error::ExpectedToken)?;
        
        // now we split on '|'
        let mut regex_cursor = line_buf.split(delimiter);
        
        // everything before '|'
        // 'match insteon-plm m'
        let head = regex_cursor.next()
            .ok_or_else(|| Error::ExpectedToken)?;
        
        // regex pattern
        // '^\x02\x60...(.).\x9b\x06$'
        let pattern = regex_cursor.next() // grabs the pattern
            .ok_or_else(|| Error::ExpectedToken)?;
        
        // everything after the second '|'
        let tail = regex_cursor.next()
            .ok_or_else(|| Error::ExpectedToken)?;
        
        // setup buffer for saving flags
        let mut flags: SmallVec<[Flags ; 2]> = SmallVec::new();
        
        // offset to enumerate where the common platform enumeration is
        let mut offset = 0;
 
        if head.len() >= 1 && pattern.len() >= 1 {
            let head = head.len()-1;
            let pattern = pattern.len()-1;

            // +2 for regex delimiters
            offset = head+pattern+2; 
        }

        else {
            return Err(Error::UnknownToken(format!("error reading line: {}", line_buf)))
        }
        
        for c in tail.chars() {
            offset += 1;
            if c == ' ' {
                break 
            }
            let flag = match c {
                'i' => Flags::IgnoreWhiteSpace,
                's' => Flags::CaseSensitive,
                unknown_flag => return Err(
                    Error::UnknownToken(format!("unknown flag ({}) found in: {}", unknown_flag, line_buf))
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
        &line_buf[offset..].chars();
    }

    else {
        // syntax error?
        // match <name> m<pattern> [<version> ...]
        return Err(
            Error::UnknownToken(format!(
                "unknown sequence expected 'm', instead got '{}' inside of '{}' ",
                 regex_char, line_buf
            ))
        )
    }
    unimplemented!()
    // MatchLineExpr {
    //     name,
    //     match_type: Directive::from_str(first_token)?,           
    // }
    // entity.patterns.push();
}
// match teamtalk m%^(?:teamtalk|welcome) userid=\d+ servername="" .* protocol="([\d.]+)"\r\nerror number=2002 message="Invalid user account"\r\n% p/BearWare TeamTalk/ i/protocol: $1/ cpe:/a:bearware:teamtalk/
