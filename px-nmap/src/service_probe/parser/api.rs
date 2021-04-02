use std::{str::FromStr, time::Duration};
use px_common::netport::PortInput;

use tokio::{
    fs::File, 
    io::{BufReader, AsyncBufReadExt}
};

use crate::error::Error;
use logos::{Lexer, Logos};

use super::{
    model::{Token, Protocol, ProbeExpr, ZeroDuration},
    match_expr::parse_match_expr
};



/// Allows our custom parsing to ask for our buffer to keep loading data. 
enum BufferPipeline {
    ContinueLoading(String),
    Continue
}

#[derive(Debug)]
pub struct FileError {
    pub cursor: Meta,
    pub error: Error,
}

impl FileError {
    pub fn new(cursor: Meta, inner: Error)  -> Self {
        Self {
            cursor,
            error: inner
        }
    }
}

#[derive(Debug, Clone)]
pub struct Meta {
    pub filepath: String,
    pub col: usize,
    pub span: std::ops::Range<usize>,
    pub use_lexer_span: bool,
}

impl Meta {
    pub fn new(name: &str) -> Self {
        Self {
            filepath: name.to_string(),
            col: 0,
            span: 0..0,
            use_lexer_span: true
        }
    }
}

impl From<(Meta, Error)> for FileError {
    fn from(x: (Meta, Error)) -> FileError {
        FileError {
            cursor: x.0,
            error: x.1
        }
    }
}

fn probe_declare_expr(line: &str, lex: &mut Lexer<Token>, expr: &mut ProbeExpr) -> Result<(), Error> {
    if expr.name.len() > 0 {
        println!("possible overwrite of previous probe entry? - [{}]", &expr.name);
    }
    
    expr.proto = match lex.next() {
        Some(Token::Word) => Protocol::from_str(lex.slice())?,
        _ => return Err(Error::ExpectedToken(Token::Word))
    };

    expr.name = match lex.next() {
        Some(Token::Word) => String::from(lex.slice()),
        _ => return Err(Error::ExpectedToken(Token::Word))
    };
    
    expr.payload = (&line[lex.span().end+1..]).trim().to_string();

    if expr.payload.len() > 0 {
        Ok(())
    }

    else {
        Err(Error::ParseError(format!("No payload detected in probe {}", &expr.name)))
    }   
}

pub async fn parse(path: &str, buf: &mut Vec<ProbeExpr>) -> Result<(), FileError> {
    let fd = File::open(path).await.unwrap();
    let mut bookkeeping = Meta::new(path);
    
    let mut fd = BufReader::new(fd);
    let mut line = String::new();
    let mut probe = ProbeExpr::default();
    //let mut i = 0;

    'read_line: loop {
        bookkeeping.col += 1;
        line.clear();
        match fd.read_line(&mut line).await {
            Ok(n) => {
                let mut trimmed = line.trim();

                'input_check: loop {   
                    let comment_flag = trimmed.starts_with("#");
                    let next_probe_flag = trimmed.contains("NEXT PROBE");
                    
                    if n == 0 { break 'read_line}
                    else if trimmed.len() == 0 { continue 'read_line }
                    else if comment_flag && !next_probe_flag { continue 'read_line }
                    else if comment_flag && next_probe_flag { 
                        buf.push(probe);
                        probe = ProbeExpr::default();
                        continue 'read_line;
                    }
                    // else if trimmed.contains("#") {
                    //     trimmed = remove_comment(trimmed);
                    //     continue 'input_check
                    // }
                    else {
                        //println!("parsed: {}", &trimmed);
                        break 'input_check
                    }
                }
                
                let mut lexer: Lexer<Token> = Lexer::new(trimmed);

                if let Err(e) = parse_line(&trimmed, &mut probe, &mut lexer, &mut bookkeeping) {
                    if bookkeeping.use_lexer_span {
                        bookkeeping.span = lexer.span();
                    }
                    return Err(FileError::new(bookkeeping, e))
                }
            }

            Err(e) => {
                return Err(FileError::new(bookkeeping, e.into()))
            }
        }
    }

    buf.push(probe);
    Ok(())
}

fn parse_line(line: &str, expr: &mut ProbeExpr, lex: &mut Lexer<Token>, meta: &mut Meta) -> Result<(), Error> {
    
    meta.use_lexer_span = true;
    let token = lex.next().unwrap();
        match token {
            Token::Probe => probe_declare_expr(&line, lex, expr)?,
            
            Token::Match => {
                meta.use_lexer_span = false;
                expr.matches.push(parse_match_expr(&line, meta)?);
            }

            Token::Rarity => expr.rarity = match lex.next() {
                Some(Token::Num) => lex.slice().parse::<u8>()?,
                 _ => return Err(Error::ExpectedToken(Token::Num))
            },

            Token::WrappedWaitMs => expr.wait_wrapped_ms = match lex.next() {
                Some(Token::Num) => ZeroDuration(Duration::from_millis(lex.slice().parse::<u64>()?)),
                 _ => return Err(Error::ExpectedToken(Token::Num))
            },
            
            Token::TotalWaitMs => expr.wait_total_ms = match lex.next() {
                Some(Token::Num) => ZeroDuration(Duration::from_millis(lex.slice().parse::<u64>()?)),
                 _ => return Err(Error::ExpectedToken(Token::Num))
            },

            Token::SslPorts => {
                while let Some(token) = lex.next() {
                    match token {
                        Token::Num | Token::Rng => { expr.tls_ports.push(PortInput::from_str(lex.slice())?); }
                        Token::Error => {}
                        _ => return Err(Error::ExpectedToken(Token::Rng))   
                    }
                }
            }

            Token::Ports => {
                while let Some(token) = lex.next() {
                    match token {
                        Token::Num | Token::Rng => { expr.tls_ports.push(PortInput::from_str(lex.slice())?); }
                        Token::Error => {}
                        _ => return Err(Error::ExpectedToken(Token::Rng))   
                    }
                }
            }
    
            Token::Exclude => {
                while let Some(token) = lex.next() {
                    match token {
                        Token::Num | Token::Rng => { expr.exclude.push(PortInput::from_str(lex.slice())?); }
                        _ => return Err(Error::ExpectedToken(Token::Rng))   
                    }
                }
            }

            Token::Fallback => match lex.next() {
                Some(Token::Word) => expr.fallback.extend(
                    lex.slice()
                        .split(",")
                        .map(|s| s.to_string())
                    ),
                 _ => return Err(Error::ExpectedToken(Token::Num))
            },

            token => return Err(Error::ParseError(format!(
                "Unexpected token at beginning of the stream, got 'Token::{:?}' ({})", token, line.split(' ').nth(0).unwrap(), 
            )))
        //}
    }
    Ok(())
}

fn remove_comment(line_buf: &str) -> &str {
    let last_idx = line_buf
        .char_indices()
        .take_while(|(i, c)| *c != '#')
        .last()
        .unwrap_or((line_buf.len()-1, '_'))
        .0;
    //dbg!(&line_buf);
    &line_buf[..last_idx]
}

#[cfg(test)]
mod test {
    use super::parse;
    use std::{
        path::PathBuf,
        str::FromStr
    };
    

    #[tokio::test]
    async fn parse_database() {
        let mut data = Vec::new();
        let mut path = PathBuf::from_str(&std::env::args().nth(0).unwrap()).unwrap();
        for _ in 0..4 {
            path.pop();
        }

        path.push("share/nmap/nmap-service-probes");
        parse(&path.to_string_lossy(), &mut data).await.unwrap();

        assert_eq!(&data.get(1).unwrap().name, "NULL");
    }

    async fn bookkeep_using_lexer() {}
    async fn bookkeep_using_meta() {}
}