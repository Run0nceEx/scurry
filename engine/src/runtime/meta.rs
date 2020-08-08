use std::time::{Duration, Instant};

const DURATION_SAMPLE_SIZE: usize = 8;

#[derive(Debug, Clone)]
pub struct CronMeta {
    pub id: uuid::Uuid,
    pub created: Instant,
    pub tts: Duration, // time to sleep
    pub ttl: Duration, // time to live
    pub ctr: usize,    // operation counter
    pub max_ctr: usize, // fail/retry counter
    pub durations: [Option<Duration>; DURATION_SAMPLE_SIZE],
    dir_ctr: usize
}

impl CronMeta {
    pub fn new(timeout: Duration, fire_in: Duration, max_retry: usize) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            created: Instant::now(),
            tts: fire_in,
            ttl: timeout,
            ctr: 0,
            max_ctr: max_retry,
            durations: [None; DURATION_SAMPLE_SIZE],
            dir_ctr: 0
        }
    }

    pub fn total_duration(&self) -> Duration {
        self.durations.iter().filter_map(|x| *x).sum()
    }

    pub fn record_elapsed(&mut self, started: Instant) {
        if self.dir_ctr >= DURATION_SAMPLE_SIZE {
            self.durations.reverse();
            self.dir_ctr = 0;
        }
        else {
            self.dir_ctr += 1;
        }
        
        self.durations[self.dir_ctr] = Some(started.elapsed());
    }

}

impl PartialEq for CronMeta {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}