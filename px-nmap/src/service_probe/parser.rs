use px_core::{
    model::PortInput,
};



use crate::model::{
    ProbeExpr,
    Protocol,
    Directive,
    Flags,
    MatchLineExpr
};

use crate::error::Error;
use std::{
    str::FromStr,
    path::PathBuf
};

use logos::{Logos, Lexer};

use smallvec::SmallVec;


fn port_input(lex: &mut Lexer<Token>) -> Option<PortInput> {
    Some(PortInput::from_str(lex.slice()).ok()?)
}


fn parse(line: &str) {
    let mut lex = Token::lexer(&line);

    match lex.next().unwrap() {
        Token::Match => {}
        Token::SoftMatch => {}
        
        Token::Probe => {}
        Token::EndProbe => {}

        Token::WrappedWaitMs => {}
        Token::TotalWaitMs => {}

        Token::SslPorts => {}
        Token::Ports => {}
        Token::Exclude => {}

        Token::Error => {}
    }
}


#[derive(Logos, Debug, PartialEq)]
enum Token {
    // Tokens can be literal strings, of any length.
    #[token("match")]
    Match,
    
    #[token("softmatch")]
    SoftMatch,

    #[token("NEXT PROBE")]
    EndProbe,

    #[token("Probe")]
    Probe,
    
    #[token("tcpwrappedms")]
    WrappedWaitMs,

    #[token("ssl_ports")]
    SslPorts,

    #[token("ports")]
    Ports,

    #[token("totalwaitms")]
    TotalWaitMs,

    #[token("Exclude")]
    Exclude,

    #[error]
    #[regex(r"[ \t\n\f\#]+", logos::skip)]
    Error,

}


// const DELIMITER: &'static str = "##############################NEXT PROBE##############################";

// fn remove_comment(line_buf: &str) -> &str {
//     let tail_position = line_buf.chars()
//         .take_while(|c| *c != '#')
//         .enumerate()
//         .map(|(i, c)| i)
//         .last()
//         .unwrap();
    
//     if tail_position >= 1 { // safety check (under-flow unsigned)
//         return &line_buf[..tail_position-1];
//     }
//     else {
//         return line_buf
//     }
// }


// pub async fn parse_txt_db(path: PathBuf, expressions: &mut Vec<ProbeExpr>, intensity: u8) -> Result<(), Error> {
//     let mut line_buf = String::new();
//     let mut entity = ProbeExpr::default();
//     // parse line by line
//     let mut row: usize = 0;

    
//     while fd.read_line(&mut line_buf)? > 0 {
//         row += 1;
//         if line_buf.len() == 0 {
//             continue
//         }

//         // if probe delimiter reached, attempt to make a `ProbeEntry` out of `ProbeExpr`
//         if line_buf.contains(&DELIMITER) {
//             if entity.name.len() > 0 && entity.payload.len() > 0 {
//                 expressions.push(entity);
//                 entity = ProbeExpr::default();
//             }
//         }
        
//         else if line_buf.trim().starts_with("#") {
//             continue
//         }
        
//         // Cut line where comment begins
//         else if line_buf.contains("#") {
//             line_buf = remove_comment(&line_buf).to_string()
//         }
        
//         // fuck me i hate parsing code ()
//         // this is how we're tokenizing
//         // bc i dont really feel like building a whole lexer+expr tree
//         let mut tokens = line_buf.split_whitespace();        
        
//         let first_token = tokens.next().ok_or_else(||Error::ExpectedToken)?;
        
//         // parses 'Probe'  
//         if first_token.eq("Probe") {
//             let protocol = tokens.next()
//                 .ok_or_else(|| Error::ExpectedToken)?;
            
//             let name = tokens.next()
//                 .ok_or_else(|| Error::ExpectedToken)?;
            
//             let payload = tokens.next()
//                 .ok_or_else(|| Error::ExpectedToken)?;

//             if entity.name.len() > 0 || entity.payload.len() > 0 {
//                 eprintln!("probe set previously and over written {} -> {}", entity.name, name);
//             }

//             entity.name = name.to_string();
//             entity.proto = Protocol::from_str(protocol)?;
//             entity.payload = payload.to_string();
//         }
        
//         // parse out 'rarity 12'
//         else if first_token.eq("rarity") {
//             entity.rarity = tokens.next()
//                 .ok_or_else(|| Error::ExpectedToken)?
//                 .parse()?;
//         }

//         // parse out 'match insteon-plm m|^\x02\x60...(.).\x9b\x06$| p/Insteon SmartLinc PLM/ i/device type: $I(1,">")/'
//         else if first_token.eq("softmatch") | first_token.eq("match") {
//             parse_match_expr(&line_buf, &first_token,  &mut tokens)?;
//         }

//         else if first_token.eq("ports") {
//             tokens.next()
//                 .ok_or_else(|| Error::ExpectedToken)?
//                 .split(',')
//                 .for_each(|port| 
//                     entity.ports.push(
//                         PortInput::from_str(port).unwrap()
//                     )
//                 );
//         }

//         else if first_token.eq("sslports") {            
//             tokens.next()
//                 .ok_or_else(|| Error::ExpectedToken)?
//                 .split(',')
//                 .for_each(|port| 
//                     entity.tls_ports.push(
//                         PortInput::from_str(port).unwrap()
//                     );
//                 );
//         }


//         else if first_token.eq("totalwaitms") {
//             entity = tokens.next().ok_or_else(|| Error::ExpectedToken)?;
//         }
//         else if first_token.eq("fallback") {
//             tokens.next().ok_or_else(|| Error::ExpectedToken)?;
//         }

//         line_buf.clear();
//     }

//     Ok(())
// }

// fn parse_ports(slice: &str, buf: &mut SmallVec<[PortInput; 16]>) {
//     slice
//         .split(',')
//         .for_each(|port| 
//             entity.tls_ports.push(
//             PortInput::from_str(port).unwrap()
//         );
//     );
// }

// fn parse_match_expr(line_buf: &str, first_token: &str, tokens: &mut std::str::SplitWhitespace) -> Result<MatchLineExpr, Error> {
//     // parsing example
//     // match insteon-plm m|^\x02\x60...(.).\x9b\x06$| p/Insteon SmartLinc PLM/ i/device type: $I(1,">")/
//     // -----
//     // match insteon-plm ...
//     // ^ we're here
//     let directive = Directive::from_str(first_token)?;

//     let name = tokens.next()
//         .ok_or_else(|| Error::ExpectedToken)?;
//     // match insteon-plm ...
//     //       ^ we're here

//     // match insteon-plm m|^\x02\x60...
//     //                   ^ we're here
//     let partial_query = tokens.next()
//         .ok_or_else(|| Error::ExpectedToken)?;
    
//     // now instead of spliting by spaces, we will just iter over the characters partially

//     let mut cursor = partial_query.chars();
    
//     // m|^\x02\x60...
//     // ^ we're here
//     let regex_char = cursor.next().ok_or_else(|| Error::ExpectedToken)?;
//     if regex_char == 'm' {
//         // m|^\x02\x60....
//         //  ^ we're here
//         let delimiter = cursor.next()
//             .ok_or_else(|| Error::ExpectedToken)?;
        
//         // now we split on '|' (delimiter)
//         let mut regex_cursor = line_buf.split(delimiter);
        
//         // everything before '|'
//         // 'match insteon-plm m'
//         let head = regex_cursor.next()
//             .ok_or_else(|| Error::ExpectedToken)?;
        
//         // regex pattern
//         // '^\x02\x60...(.).\x9b\x06$'
//         let pattern = regex_cursor.next() // grabs the pattern
//             .ok_or_else(|| Error::ExpectedToken)?;
        
//         // everything after the second '|'
//         let tail = regex_cursor.next()
//             .ok_or_else(|| Error::ExpectedToken)?;
        
//         // setup buffer for saving flags
//         let mut flags: SmallVec<[Flags ; 2]> = SmallVec::new();
        
//         // offset to enumerate where the common platform enumeration is
//         let mut offset = 0;
 
//         if head.len() >= 1 && pattern.len() >= 1 {
//             let head = head.len()-1;
//             let pattern = pattern.len()-1;

//             // +2 for regex delimiters
//             offset = head+pattern+2; 
//         }

//         else {
//             return Err(Error::UnknownToken(format!("error reading line: {}", line_buf)))
//         }
        
//         for c in tail.chars() {
//             offset += 1;
//             if c == ' ' {
//                 break 
//             }
//             let flag = match c {
//                 'i' => Flags::IgnoreWhiteSpace,
//                 's' => Flags::CaseSensitive,
//                 unknown_flag => return Err(
//                     Error::UnknownToken(format!("unknown flag ({}) found in: {}", unknown_flag, line_buf))
//                 )
//             };

//             if !flags.contains(&flag) {
//                 flags.push(flag)
//             }
//         }
//         //  p/Android Debug Bridge/
//         //  i/auth required: $I(1,"<")/
//         //  o/Android/
//         //  cpe:/o:google:android/a
//         //  cpe:/o:linux:linux_kernel/a
//         &line_buf[offset..].chars();
//     }

//     else {
//         // syntax error?
//         // match <name> m<pattern> [<version> ...]
//         return Err(
//             Error::UnknownToken(format!(
//                 "unknown sequence expected 'm', instead got '{}' inside of '{}' ",
//                  regex_char, line_buf
//             ))
//         )
//     }
//     MatchLineExpr {
//         name,
//         match_type: Directive::from_str(first_token)?,           
//     }
// }