// welcome to the junk pile

mod enumerations;
pub use enumerations::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortInput {
    Singleton(u16),
    Range(std::ops::Range<u16>)
}
