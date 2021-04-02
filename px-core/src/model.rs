
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum State {
    Closed,
    Filtered,
    Open,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            State::Closed => "closed",
            State::Open => "open",
            State::Filtered => "filtered"
        };
        
        write!(f, "{}", x)?;
        Ok(())
    }
}
