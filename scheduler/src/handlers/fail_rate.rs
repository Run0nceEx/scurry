use crate::{
    schedule::{
        SignalControl,
        meta::CronMeta,
        sugar::MetaSubscriber
    },
    error::Error,
};


#[derive(Debug)]
pub struct WarnConstFailRate {
    threshold_percentage: f32,
    threshold_limiter: usize,       // Amount to buffer before precentages
    pub positive_cnt: usize,
    pub negative_cnt: usize
}

// a simple logger to check if a schedule is failing consistently
impl WarnConstFailRate {
    pub fn new(threshold: (f32, usize)) -> Self {
        Self {
            threshold_percentage: threshold.0,
            threshold_limiter: threshold.1,
            positive_cnt: 0,
            negative_cnt: 0
        }
    }

    pub fn percentage(&self) -> f32 {
        ((self.positive_cnt+self.negative_cnt) as f32 * 0.01) / self.negative_cnt as f32 * 0.01
    }
}

#[async_trait::async_trait]
impl MetaSubscriber for WarnConstFailRate {
    async fn handle(&mut self, meta: &mut CronMeta, signal: &mut SignalControl) -> Result<(), Error> {
        match signal {
            SignalControl::Success => self.positive_cnt += 1,
            SignalControl::Drop | SignalControl::Fuck => self.negative_cnt += 1,
            _ => {}
        }
        
        if self.positive_cnt+self.negative_cnt >= self.threshold_limiter {
            let percentage = self.percentage();
            if percentage >= self.threshold_percentage {
                println!("{} is failing at {} rate", meta.handler_name, percentage)
            }
        }
        Ok(())
    }
}
