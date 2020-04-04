// Single Collector
pub trait Metric {
    fn calc_score(&self) -> f32;
    fn get_weight(&self) -> f32;
}

pub struct ScoreData {
    data_points: Vec<Box<dyn Metric>>,
    weight: f32
}

impl Metric for ScoreData {
    fn get_weight(&self) -> f32 {
        self.weight
    }

    fn calc_score(&self) -> f32 {
        let scores: Vec<f32> = self.data_points.iter().map(|x| x.calc_score()*x.get_weight()).collect();
        scores.iter().sum::<f32>() / scores.len() as f32
    }
}
