mod cpe;
mod match_expr;
mod version_info;

pub mod model;
pub use match_expr::{MatchLineExpr, parse_match_expr};

use std::{
    time::Duration,
    str::FromStr
};

use tokio::{
    fs::File, 
    io::{BufReader, AsyncBufReadExt}
};

use crate::error::Error;
use logos::{Lexer, Logos};
use px_core::model::PortInput;
pub use model::{Directive, Token, Protocol, ProbeExpr, ZeroDuration};

#[derive(Debug, )]
struct FileError {
    cursor: Meta,
    error: Error,
}

impl FileError {
    fn new(cursor: Meta, inner: Error)  -> Self {
        Self {
            cursor,
            error: inner
        }
    }
}

#[derive(Debug, Clone)]
struct Meta {
    filepath: String,
    col: usize,
}

impl Meta {
    pub fn new(name: &str) -> Self {
        Self {
            filepath: name.to_string(),
            col: 0,
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

#[inline]
fn handle_err<T, E>(data: Result<T, E>, meta: &Meta) -> Result<T, FileError>
where E: Into<Error> {
    match data {
        Ok(x) => return Ok(x),
        Err(e) => return Err(FileError::new(meta.clone(), e.into()))
    }
}

pub async fn parse(path: &str, buf: &mut Vec<ProbeExpr>) -> Result<(), FileError> {
    let mut meta = Meta::new(path);

    let fd = handle_err(File::open(path).await, &meta)?;
    let mut fd = BufReader::new(fd);
    let mut line = String::new();
    let mut probe = ProbeExpr::default();

    
    while handle_err(fd.read_line(&mut line).await, &meta)? > 0 {
        if let Ok(true) = parse_line(&line, &mut probe,  &mut meta) {
            buf.push(probe);
            probe = ProbeExpr::default();
        }
    }
    buf.push(probe);
    
    Ok(())
}

fn parse_line(line: &str, expr: &mut ProbeExpr, meta: &mut Meta) -> Result<bool, Error> {
    let line = remove_comment(line);

    let mut lex = Token::lexer(&line);

    if let Some(token) = lex.next() {
        match token {
            Token::Match => expr.matches.push(parse_match_expr(&line, &mut lex)?),
            Token::Probe => probe_declare_expr(&line, &mut lex, expr)?,
            
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
                        Token::Error => {}
                        _ => return Err(Error::ExpectedToken(Token::Rng))   
                    }
                }
            }

            Token::Error => {}
            Token::Num   => return Ok(false),
            Token::Rng   => return Ok(false),
            Token::Word  => return Ok(false),
            Token::EndProbe => return Ok(true),
        }
        meta.col += 1;
    }

    Ok(false)
}

fn remove_comment(line_buf: &str) -> &str {
    let tail_position = line_buf.chars()
        .take_while(|c| *c != '#')
        .enumerate()
        .map(|(i, c)| i)
        .last()
        .unwrap();
    
    if tail_position >= 1 { // safety check (under-flow unsigned)
        return &line_buf[..tail_position-1];
    }
    else {
        return line_buf
    }
}

#[cfg(test)]
mod test {
    use super::parse;

    #[tokio::test]
    async fn no_error() {
        let mut data = Vec::new();
        assert!(match parse("/usr/share/nmap/nmap-service-probes", &mut data).await {
            Ok(()) => true,
            Err(e) => { eprintln!("{:?}", e); false}
        });
    }
}