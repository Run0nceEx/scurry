// Single Collector

pub trait CalcScore {
    fn calc_score(&self) -> f32;
}

// probably faster calcs if i use nalgerbra
// but maybe later

pub struct Score {
    weight: f32,
    bias: f32,
    score: f32
}

impl CalcScore for Score {
    fn calc_score(&self) -> f32 {
        self.weight * (self.score + self.bias)
    }
}

impl CalcScore for Vec<Score> {
    fn calc_score(&self) -> f32 {
        self.iter().map(|s| s.calc_score()).sum::<f32>() / self.len() as f32
    }
}

pub enum Error {}

pub trait Judge {
    // setup a service if needed
    fn setup() -> Result<(), Error> { Ok(()) }

    // Run score system
    fn judge<T, S>(&self, protocol: T) -> S
    where 
        T: Protocol + CalcScore;
}

