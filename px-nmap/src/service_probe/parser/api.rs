use std::{str::FromStr, time::Duration, unimplemented};

use tokio::{
    fs::File, 
    io::{BufReader, AsyncBufReadExt}
};

use crate::error::Error;
use logos::{Lexer, Logos};
use px_core::model::PortInput;
use super::{
    model::{Token, Protocol, ProbeExpr, ZeroDuration},
    match_expr::parse_match_expr
};

#[derive(Debug)]
pub struct FileError {
    cursor: Meta,
    error: Error,
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
    pub span: std::ops::Range<usize>
}

impl Meta {
    pub fn new(name: &str) -> Self {
        Self {
            filepath: name.to_string(),
            col: 0,
            span: 0..0
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
    let fd = File::open(path).await.unwrap();
    let mut bookkeeping = Meta::new(path);
    
    let mut fd = BufReader::new(fd);
    let mut line = String::new();
    let mut probe = ProbeExpr::default();
    
    loop {
        match fd.read_line(&mut line).await {
            Ok(n) => {
                if n == 0 { break }
                else { 
                    let line = remove_comment(line.as_str());
                    let mut lexer: Lexer<Token> = Lexer::new(line);

                    match parse_line(&line, &mut probe, &mut lexer) {
                        Ok(ready) => {
                            if ready {
                                buf.push(probe);
                                probe = ProbeExpr::default();
                            }
                        }
                        Err(e) => {
                            bookkeeping.span = lexer.span();
                            return Err(FileError::new(bookkeeping, e))
                        }
                    }
                }
                bookkeeping.col += 1;
            }

            Err(e) => {
                return Err(FileError::new(bookkeeping, e.into()))
            }
        }
    }

    buf.push(probe);
    Ok(())
}

fn parse_line(line: &str, expr: &mut ProbeExpr, lex: &mut Lexer<Token>) -> Result<bool, Error> {
    let line = remove_comment(line);

    if let Some(token) = lex.next() {
        match token {
            Token::Probe => probe_declare_expr(&line, lex, expr)?,
            Token::Match => expr.matches.push(parse_match_expr(&line)?),
            
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
    }

    Ok(false)
}

fn remove_comment(line_buf: &str) -> &str {
    let tail_position = line_buf
        .char_indices()
        .take_while(|c| c.1 != '#')
        .last()
        .unwrap()
        .0;
    
    let pound_sign_amount = line_buf[tail_position..]
        .char_indices()
        .take_while(|(_, c)| *c == '#')
        .last()
        .unwrap()
        .0;
    
    if pound_sign_amount == 30 || tail_position == 0 {
        // "###... NEXT PROBE ###..."
        return line_buf
    }

    return &line_buf[..tail_position-1];
}

#[cfg(test)]
mod test {
    use super::parse;

    // #[tokio::test]
    // async fn no_error() {
    //     let mut data = Vec::new();
    //     assert!(match parse("/usr/share/nmap/nmap-service-probes", &mut data).await {
    //         Ok(()) => true,
    //         Err(e) => { eprintln!("{:?}", e); false}
    //     });
    // }
}