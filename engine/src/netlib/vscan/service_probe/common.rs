// input & output structures for module
// these should be the only data structures 


use super::parser::Directive;

#[derive(Default, Debug, Clone)]
pub struct CPE {
    part: String,
    vendor: String,
    product: String,
    OS: String,
}
