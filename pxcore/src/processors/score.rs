// Single Collector
//use crate::{Scannable, Connector};

pub trait CalcScore {
    fn score(&self) -> f32;
}

// probably faster calcs if i use nalgerbra
// but maybe later
pub struct Score {
    weight: f32,
    bias: f32,
    real_score: f32,
}

impl CalcScore for Score {
    fn score(&self) -> f32 {
        self.weight * (self.real_score + self.bias)
    }
}

impl CalcScore for Vec<Score> {
    fn score(&self) -> f32 {
        self.iter().map(|s| s.score()).sum::<f32>() / self.len() as f32
    }
}

// /// Attempt unit scoring on service
// pub trait Puppet<C, R>  { 
//     fn run_puppet(&self) -> R
//     where 
//         Self: Scannable<C>,
//         C: Connector, 
//         R: CalcScore;
// }


