use crate::error::Error;
use super::model::DataField;
use std::str::FromStr;

struct Field {
    ident: Identifier,
    data: String
}

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
pub struct CPExpr {
    pub part: Identifier,
    pub product_name: Option<DataField>,
    pub version: Option<DataField>,
    pub vendor: Option<DataField>,
    pub info: Option<DataField>,
    pub update: Option<DataField>,
    pub edition: Option<DataField>,
    pub language: Option<DataField>,
}
