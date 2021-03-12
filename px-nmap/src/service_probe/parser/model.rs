use std::{borrow::Borrow, str::FromStr, time::Duration};
use bincode::Options;

use px_core::model::PortInput;
use logos::{Logos, Lexer};

use crate::error::Error;
use super::{
    cpe::Identifier,
    match_expr::MatchLineExpr,
};


#[derive(Debug)]
pub struct ZeroDuration(pub Duration);
impl Default for ZeroDuration {
    fn default() -> Self {
        Self(Duration::from_millis(0))
    }
}

impl ZeroDuration {
    pub fn into_inner(self) -> Duration {
        self.0
    }
}

#[derive(Default, Debug)]
pub struct ProbeExpr {
    pub proto: Protocol,
    pub payload: String,
    pub payload_delimiter: char,
    pub rarity: u8,
    pub load_ord: usize,
    pub name: String,
    pub ports: Vec<PortInput>,
    pub exclude: Vec<PortInput>,
    pub tls_ports: Vec<PortInput>,
    pub matches: Vec<MatchLineExpr>,
    pub fallback: Option<String>,
    pub wait_total_ms: ZeroDuration,
    pub wait_wrapped_ms: ZeroDuration,
}

#[derive(Logos, Debug, PartialEq, Copy, Clone)]
pub enum Token {
    // Tokens can be literal strings, of any length.
    #[token("softmatch")]
    #[token("match")]
    Match,
    
    #[regex("[# ]*NEXT PROBE[# ]*")]
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

    #[regex("[0-9]+-[0-9]+")]
    Rng,

    #[regex("[0-9]+", priority = 2)]
    Num,

    #[regex("[a-zA-Z0-9]+", priority = 3)]
    Word,
    
    #[error]
    #[regex(r"[\t\n\f\r ]+", logos::skip)]
    Error,
}
// const DELIMITER: &'static str = "##############################NEXT PROBE##############################";

#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    TCP,
    UDP
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::TCP
    }
}

impl FromStr for Protocol {
    type Err = Error;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x.trim() {
            "udp" | "UDP" => Protocol::UDP,
            "tcp" | "TCP" => Protocol::TCP,
            x => return Err(
                Error::ParseError(format!("Got \"{}\" instead of 'tcp' or 'udp'", x))
            ) 
        })
    }
}


#[derive(Debug, Copy, Clone)]
pub enum Directive {
    SoftMatch,
    Match
}


impl FromStr for Directive {
    type Err = Error;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        Ok(match x.trim() {
            "softmatch" => Directive::SoftMatch,
            "match" => Directive::Match,
            _ => return Err(
                Error::ExpectedToken(Token::Match)
            ) 
        })
    }
}


// Fields allow Expressions/functions to occur

enum EndianSymbol {
    Big,
    Little
}

impl std::str::FromStr for EndianSymbol {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sym = match s {
            ">" => EndianSymbol::Big,
            "<" => EndianSymbol::Little,
            _ => return Err(Error::ParseError(format!("expected '<', or '>' but got '{}'", s)))
        };
        Ok(sym)
    }
}

enum HelperFunction {
    /// $P()
    /// Filters out unprintable characters. 
    /// This is useful for converting Unicode UTF-16 encoded strings such as W\0O\0R\0K\0G\0R\0O\0U\0P\0
    /// into the ASCII approximation WORKGROUP. 
    /// It can be used in any versioninfo field by passing it the number of the match you want to make printable,
    /// like this: i/$P(3)/.
    Print(usize),

    /// $SUBST()
    /// Makes substitutions in matches before they are printed.
    /// It takes three arguments.
    /// The first is the substitution number in the pattern, just as you would use in a normal replacement variable such as $1 or $3.
    /// The second and third arguments specify a substring you wish to find and replace, respectively.
    /// All instances of the match string found in the substring are replaced, not just the first one.
    /// For example, the VanDyke VShell sshd gives its version number in a format such as 2_2_3_578.
    /// We use the versioninfo field v/$SUBST(1,"_",".")/ to convert it to the more conventional form 2.2.3.578.
    Substitute(usize, String, String),

    /// $I()
    /// Unpacks an unsigned integer from the captured bytes.
    /// Given a captured string of up to 8 bytes, 
    /// it will treat them as an unsigned integer and convert it to decimal format.
    /// It takes two arguments. The first is the substitution number in the pattern.
    /// The second is the string ">" to indicate that the bytes are in big-endian order, or "<" to indicate little-endian.
    UnpackInt(usize, EndianSymbol)
}
impl HelperFunction {
    fn new(data: &str) -> Result<Self, Error> {
        let mut chars = data.chars();
        let first_char = chars.nth(1).unwrap();

        let raw_arg_data: String = chars.skip_while(|c| *c != '(').take_while(|x| *x != ')').collect();
        let mut arguements = raw_arg_data.split(',');

        let func = match first_char {
            'i' | 'I' => Self::UnpackInt(
                arguements.next().unwrap().parse()?,
                arguements.next().unwrap().parse()?
            ),

            's' | 'S' => Self::Substitute(
                arguements.next().unwrap().parse()?,
                arguements.next().unwrap().to_string(),
                arguements.next().unwrap().to_string()
            ),
            'p' | 'P' => Self::Print(
                arguements.next().unwrap().parse()?,
            ),
            _ => return Err(Error::ParseError(format!("Expected $I, $SUBST, or $P, got '{}'", data)))
        };
        
        Ok(func)
    }

    fn run(&self, matches: &[regex::bytes::Match]) -> Result<String, Error> {
        let string = match self {
            HelperFunction::Print(idx) => 
                String::from_utf8_lossy(&matches[*idx].as_bytes()).replace("�", ""),
            
            HelperFunction::Substitute(idx, original, replacement) =>
                String::from_utf8_lossy(&matches[*idx].as_bytes()).replace(original, replacement),
            
             HelperFunction::UnpackInt(index, endianness) => {
                let serializer = bincode::DefaultOptions::new()
                    .with_fixint_encoding();
                
                let num: u64 = match endianness {
                    EndianSymbol::Big => { 
                        serializer.with_big_endian().deserialize(&matches[*index].as_bytes())?
                    },
                    EndianSymbol::Little => {
                        serializer.with_little_endian().deserialize(&matches[*index].as_bytes())?
                    }
                };

                format!("{}", num)
            }
        };

        Ok(string)
    }
}

#[derive(Logos, Debug, PartialEq)]
enum InterpretToken {
    //$1 $12 $143
    #[regex("[$][0-9]+")]
    MatchNth,

    //$FOO() $bar(boo, beez)
    #[regex("[$][a-zA-Z]+(.+|)")]
    Func,

    #[error]
    #[regex("\t\r\n ", logos::skip)]
    Error
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataField {
    schematic: String,
}

impl DataField {
    pub fn interpret(&self, matches: &[regex::bytes::Match]) -> Result<String, Error>{
        let mut ret = String::new();
        let mut lexer = InterpretToken::lexer(&self.schematic);
        let mut last = 0;

        while let Some(token) = lexer.next() {
            let span = lexer.span();
            // places head from self.schematic into ret
            ret.push_str(&self.schematic[last..span.start]);
            last = span.end;
            match token {
                InterpretToken::MatchNth => {
                    let idx: usize = lexer.slice()[1..].parse()?;
                    let replacement = &matches[idx];
                    ret.push_str(&String::from_utf8_lossy(replacement.as_bytes()));
                }

                InterpretToken::Func => {
                    ret.push_str(&HelperFunction::new(lexer.slice())?.run(&matches)?);
                }

                InterpretToken::Error => continue
            }
        }

        ret.push_str(&self.schematic[last..]);

        Ok(ret)
    }

    pub fn new(inner: String) -> Self {
        Self {
            schematic: inner
        }
    }
}


impl From<String> for DataField {
    fn from(x: String) -> DataField {
        DataField::new(x)
    }
}


impl From<&str> for DataField {
    fn from(x: &str) -> DataField {
        DataField::new(x.to_string())
    }
}