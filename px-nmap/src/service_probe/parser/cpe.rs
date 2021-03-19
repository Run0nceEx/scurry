use crate::error::Error;
use super::model::DataField;
use std::{str::FromStr, unimplemented};

// [independent_Fieldibute]
// p/Clementine music player remote control/
// <ident><delimiter>String Field<delimiter>

// Common Platform Enum
// cpe:/a:clementine-player:clementine/
// cpe:/<ident>:<vendor>:<product>:<version>:<update>:<edition>:<language>
// cpe:/<ident>:<Field>:...

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

    pub fn interpret(&self, matches: &[regex::bytes::Match]) -> Result<CPE, Error> {
        self.0.interpret(matches)?.parse()
    }
}

// cpe:/<part>:<vendor>:<product>:<version>:<update>:<edition>:<language>
struct CPE {
    pub part: Identifier,
    pub vendor: Option<String>,
    pub product: Option<String>,
    pub version: Option<String>,
    pub update: Option<String>,
    pub edition: Option<String>,
    pub language: Option<String>
}

impl FromStr for CPE {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        unimplemented!()
    }
}