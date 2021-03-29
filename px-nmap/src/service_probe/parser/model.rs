use std::{
    borrow::Borrow,
    str::{FromStr},
    time::Duration
};
use regex::{bytes::Match, bytes::Matches};
use bincode::Options;

use px_core::model::PortInput;
use logos::{Logos, Lexer};

use crate::error::Error;
use super::{
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
    #[token("softmatch")]
    #[token("match")]
    Match,

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

    #[token("Exclude T:")]
    Exclude,

    #[regex("[0-9]+-[0-9]+")]
    Rng,

    #[regex("[0-9]+", priority = 2)]
    Num,

    #[regex("[a-zA-Z0-9]+", priority = 3)]
    Word,
    
    #[error]
    #[regex(r"[\t\n\f\r ,]+", logos::skip)]
    Error,
}

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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

        let raw_arg_data: String = chars.skip_while(|c| *c != '(').skip(1).take_while(|x| *x != ')').collect();
        let mut arguements = raw_arg_data.split(',');
        
        let idx_correct = |i: usize| {
            if i > 0 { Ok(i-1) } 
            else {
                return Err(Error::ParseError("Selector index cannot be 0".to_string())) 
            }
        };
    
        let clean_quotes = |s: &str| {
            if s.contains("\\\"") { // person wants to replace the `"` character
                "\"".to_string()
            }
            else if s.contains("\\'") { // person wants to replace the `'` character
                "'".to_string()
            }
            else {  // deal with everything else
                s.replace("\"", "").replace("\\'", "")
            }
        };

        let func = match first_char {
            'i' | 'I' => Self::UnpackInt(
                idx_correct(arguements.next().unwrap().parse()?)?,
                clean_quotes(arguements.next().unwrap()).parse()?
            ),

            's' | 'S' => Self::Substitute(
                idx_correct(arguements.next().unwrap().parse()?)?,
                clean_quotes(&arguements.next().unwrap().to_string()),
                clean_quotes(&arguements.next().unwrap().to_string())
            ),
            'p' | 'P' => Self::Print(
                idx_correct(arguements.next().unwrap().parse()?)?
            ),
            _ => return Err(Error::ParseError(format!("Expected $I, $SUBST, or $P, got '{}'", data)))
        };
        
        Ok(func)
    }

    fn run(&self, matches: &[Match<'_>]) -> Result<String, Error> {
        let string = match self {
            HelperFunction::Print(idx) => 
                String::from_utf8_lossy(
                    matches.get(*idx)
                        .unwrap()
                        .as_bytes()
                )
                    .replace("�", "")
                    .chars()
                    .filter(|c| (*c as u8) > 31 && 128 > (*c as u8)).collect(),
            
            HelperFunction::Substitute(idx, original, replacement) =>
                String::from_utf8_lossy(matches.get(*idx).unwrap().as_bytes()).replace(original, replacement),
            
             HelperFunction::UnpackInt(index, endianness) => {
                let serializer = bincode::DefaultOptions::new()
                    .with_fixint_encoding();
                
                let num: u64 = match endianness {
                    EndianSymbol::Big => { 
                        serializer.with_big_endian().deserialize(dbg!(matches.get(*index).unwrap().as_bytes()))?
                    },
                    EndianSymbol::Little => {
                        serializer.with_little_endian().deserialize(dbg!(matches.get(*index).unwrap().as_bytes()))?
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
    pub fn interpret(&self, matches: &[Match<'_>]) -> Result<String, Error> {
        let mut ret = String::new();
        let mut lexer = InterpretToken::lexer(&self.schematic);
        let mut last = 0;

        while let Some(token) = lexer.next() {
            let span = lexer.span();
            // places head from self.schematic into ret
            ret.push_str(&self.schematic[last..span.start]);
            dbg!(&ret);
            dbg!(&token);
            dbg!(&lexer.slice());
            
            last = span.end;

            // TODO(ADAM)
            // okay comment for future self
            // every selector index (the first usize field of each `HelperFunction` - including `InterpretToken::MatchNth`) 
            // all require we subtract 1 from the selector index (idx -= 1) the index integer)
            // right now, we're doing this by hand
            // but it makes more sense to instrument a structure for this at some point
            // so we dont have a scatter brain of code here

            match token {
                InterpretToken::MatchNth => {
                    // "$1" -> 1: usize
                    let mut idx: usize = lexer.slice()[1..].parse()?;
                    if idx == 0 {
                        return Err(Error::ParseError(
                            "match selector index cannot be zero ($0)".to_string()
                        ))
                    }
                    idx -= 1;

                    let selected = matches.get(idx);
                    if let None = selected {
                        return Err(Error::ParseError(format!(
                            "match selected not found (${})", idx
                        )))
                    }

                    let data = String::from_utf8_lossy(selected.unwrap().as_bytes());
                    ret.push_str(&data);
                }

                InterpretToken::Func => {
                    // each helper func handles index correction its self
                    let data = HelperFunction::new(lexer.slice())?
                        .run(matches)?;
                    ret.push_str(data.as_str());
                }
                
                InterpretToken::Error => ret.push_str(lexer.slice())
                
            }
        }

        ret.push_str(&self.schematic[last..]);
        Ok(ret)
    }

    pub fn new(inner: &str) -> Self {
        let mut string = String::from(inner);
        string.shrink_to_fit();
        Self {
            schematic: string
        }
    }

    pub fn into_inner(self) -> String {
        self.schematic
    }
}

impl From<String> for DataField {
    fn from(x: String) -> DataField {
        DataField::new(x.as_str())
    }
}


impl From<&str> for DataField {
    fn from(x: &str) -> DataField {
        DataField::new(x)
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Identifier {
    Application,
    Hardware,
    OperationSystem
}


impl FromStr for Identifier {
    type Err = Error;

    fn from_str(c: &str) -> Result<Self, Self::Err> {
        Ok(match c {
            "a" => Identifier::Application,
            "h" => Identifier::Hardware,
            "o" => Identifier::OperationSystem,
            c => return Err(
                Error::ParseError(format!("Expected char 'a', 'h', or 'o', got '{}'", c))
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CPExpr(DataField);
impl CPExpr {
    pub fn new(field: DataField) -> Self {
        Self(field)
    }

    pub fn interpret(&self, matches: &[Match]) -> Result<CPE, Error> {
        self.0.interpret(matches)?.parse()
    }

    pub fn into_inner(self) -> String {
        self.0.into_inner()
    }
}

// cpe:/<part>:<vendor>:<product>:<version>:<update>:<edition>:<language>
#[derive(Clone, Debug)]
struct CPE {
    pub part: Identifier,
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub update: Option<String>,
    pub edition: Option<String>,
    pub language: Option<String>
}

impl CPE {
    fn new(ident: Identifier) -> Self {
        Self {
            part: ident,
            vendor: None,
            product: None,
            version: None,
            update: None,
            edition: None,
            language: None
        }
    }
}

#[inline]
fn len_chk(seg: &str) -> Option<String> {
    if seg.len() > 0 { return Some(seg.to_string()) }
    else { return None }
}

impl FromStr for CPE {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cpe = CPE::new(Identifier::Application);
        for (i, seg) in s[3..].split(':').enumerate() {
            match i {
                0 => cpe.part = Identifier::from_str(&s)?,
                1 => cpe.vendor = len_chk(seg),
                2 => cpe.product = len_chk(seg),
                3 => cpe.version = len_chk(seg),
                4 => cpe.update = len_chk(seg),
                5 => cpe.edition = len_chk(seg),
                6 => cpe.language = len_chk(seg),
                _ => return Err(
                    Error::ParseError("Got too many segments in CPE expression (cpe:/:<seg>:<seg>.../)".to_string())
                )
            }
        }
        Ok(cpe)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    
    #[test]
    fn intrepret_datafield_index() {
        let field = DataField::new("hello $1");
        let pattern = regex::bytes::Regex::new(r"(.*)").unwrap();
        let matches: Vec<Match> = pattern.find_iter(b"adam").collect();

        let constructed = field.interpret(&matches[..]).unwrap();
        
        assert_eq!(constructed, "hello adam")
    }

    #[test]
    fn intrepret_datafield_print() {
        let field = DataField::new(r"hello $P(1)");
        
        let pattern = regex::bytes::Regex::new(r"(.*)").unwrap();
        let matches: Vec<Match> = pattern.find_iter(b"first_match\x00\x00\x00").collect();

        let constructed = field.interpret(&matches[..]).unwrap();
        assert_eq!(constructed, "hello first_match")
    }


    #[test]
    fn intrepret_datafield_substitute() {
        let field = DataField::new("hello $SUBST(1,\"_\",\".\")");
        let pattern = regex::bytes::Regex::new(r"(.*)").unwrap();
        let matches: Vec<Match> = pattern.find_iter(b"1_1_1_1").collect();

        let constructed = field.interpret(&matches[..]).unwrap();
        
        assert_eq!(constructed, "hello 1.1.1.1")
    }

    #[test]
    fn intrepret_datafield_unpack_int() {
        let field = DataField::new("hello $I(1,\"<\")");
        let pattern = regex::bytes::Regex::new(r"(.*)").unwrap();

        let matches: Vec<Match> = pattern.find_iter(&[0;8]).collect();
        let constructed = field.interpret(&matches[..]).unwrap();
        assert_eq!(constructed, "hello 0")
    }
}