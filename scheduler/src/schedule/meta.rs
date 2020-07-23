use std::time::{Duration, Instant};
use smallvec::SmallVec;


#[derive(Debug, Clone)]
pub struct CronMeta {
    pub id: uuid::Uuid,
    pub created: Instant,
    pub tts: Duration, // time to sleep
    pub ttl: Duration, // time to live
    pub ctr: usize,    // operation counter
    pub max_ctr: usize, // fail/retry counter
    pub durations: SmallVec<[Duration; 8]>,
    pub handler_name: String,
}


impl CronMeta {
    pub fn new(timeout: Duration, fire_in: Duration, handler: String, max_retry: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created: Instant::now(),
            tts: fire_in,
            ttl: timeout,
            ctr: 0,
            max_ctr: max_retry,
            durations: SmallVec::new(),
            handler_name: handler
        }
    }

    pub fn avg_duration(&self) -> Duration {
        let mut i = 0;
        let mut total = Duration::from_secs(0);

        for time in self.durations.iter() {
            total += *time;
            i +=1 ;
        }

        total/i
    }

    pub fn total_duration(&self) -> Duration {
        self.durations.iter().sum()
    }
}
