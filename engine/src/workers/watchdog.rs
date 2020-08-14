/*
watchdog.rs
----------

WatchDog attempts to accomplish minor maintence issues such as,
    *checking networking connectivity
    *suspecious fail/success rates, and short/long execution durations

by sampling the results on the increment of `SAMPLE_EVERY`
and using that data to monitor activity

*/

use crate::{
    runtime::{
        SignalControl,
        meta::CronMeta,
        //pool::MetaSubscriber
    },
    error::Error,
};

use std::{
    time::{Duration, Instant},
};

use tokio::net::TcpStream;


const HOSTLIST: &'static [&'static str] = &[
    // Quad9
    "9.9.9.9",

    // OpenDNS
    "208.67.222.222",
    "208.67.220.220",

    // googleDNS
    "8.8.8.8",
    "8.8.4.4",

    // cloudflare DNS
    "1.1.1.1",
    "1.0.0.1",

    //Freenon
    "80.80.80.80",
    "80.80.81.81",

];


#[derive(Debug)]
pub enum NetworkState {
    /// Everything seems to be working
    Ok,

    /// Assumes we can't reach anything 
    Failure {at: Instant, next_check: (Instant, Duration)},
    
    /// We've irritated the target, and we believe they've blocked us
    TargetBlocking,
    
    /// We believe that a failure has occured, 
    /// and this state will be temporarily kept
    /// until investigations are resolved 
    Sampling(JobPoolSample)
}


#[derive(Debug, Clone, Copy)]
pub struct JobPoolSample {
    sample_size: usize,
    duration_bounds: (Duration, Duration),
    fail_bounds: (f32, f32),
    neg: usize,
    pos: usize,
    created: Instant,
    total_duration: Duration
}


impl JobPoolSample {
    pub fn new(sample_size: usize, fail_bounds: (f32, f32), duration_bounds: (Duration, Duration)) -> Self {
        assert!(fail_bounds.0 <= fail_bounds.1);
        assert!(duration_bounds.0 <= duration_bounds.1);

        Self {
            created: Instant::now(),
            neg: 0,
            pos: 0,
            total_duration: Duration::from_secs(0),
            fail_bounds,
            duration_bounds,
            sample_size,
        }
    }

    pub fn percentage(&self) -> f32 {
        ((self.pos+self.neg) as f32 * 0.01) / self.pos as f32 * 0.01
    }

    pub fn avg_time(&self) -> Duration {
        Duration::from_secs(self.total_duration.as_secs() / self.sample_size as u64)
    }
}


/// Checks if we're connected by some hosts
pub async fn www_available() -> bool {
    for host in HOSTLIST {
        match TcpStream::connect(format!("{}:53", host)).await {
            Ok(_) => return true,
            Err(e) => eprintln!("{}", e)
        }
    }

    return false
}

const SAMPLES_TAKEN: usize = 5;
const SAMPLE_SIZE: usize = 100;

#[derive(Debug)]
pub struct WatchDog {
    state: NetworkState,        
    //samples: SmallVec<[JobPoolSample; SAMPLES_TAKEN]>,
    sample_ctr: usize,
}

// a simple logger to check if a schedule is failing consistently
impl WatchDog {
    pub async fn new() -> Self {
        let net = {
            if www_available().await {
                NetworkState::Ok
            } else {
                NetworkState::Failure {
                    at: Instant::now(),
                    next_check: (Instant::now(), Duration::from_secs(10))
                }
            }
        };

        Self {
            state: net,
            //samples: SmallVec::new(),
            sample_ctr: 0,
        }
    }
}

const SAMPLE_EVERY: usize = 40000;

// #[async_trait::async_trait]
// impl MetaSubscriber for WatchDog {
//     async fn handle(&mut self, meta: &mut CronMeta, signal: &SignalControl) -> Result<SignalControl, Error> {

//         match &self.state {
//             NetworkState::Failure{next_check: (last, tts), at} => {
//                 if last.elapsed() >= *tts {
//                     if www_available().await {
//                         eprintln!(
//                             "Recovered from network outage at {:?} that lasted {} seconds", 
//                             at, at.elapsed().as_secs()
//                         );
//                         self.state = NetworkState::Ok;
//                     }
//                     else {
//                         self.state = NetworkState::Failure{at: *at, next_check: (Instant::now(), meta.tts)}
//                     }
//                 }

//                 return Ok(SignalControl::Retry);
//             }

//             NetworkState::Sampling(mut sample) => {
//                 match signal {
//                     SignalControl::Success(connected) => {
//                         if *connected {
//                             sample.pos += 1;
//                         } else {
//                             sample.neg += 1;
//                         }
//                     },
                    
//                     SignalControl::Drop => sample.neg += 1,
//                     _ => {} 
//                 }
//                 // Use the time of execution as an indicator of failure
//                 sample.total_duration += meta.total_duration();

//                 // if its reached its bottom limit, we can start analyzing what we have
//                 if sample.pos + sample.neg >= sample.sample_size {
//                     let flags = check_sample(&sample);
//                     if (flags.fishy_negative_rate || flags.fishy_positive_rate) 
//                     && (flags.fishy_low_lifetime  || flags.fishy_high_lifetime) {
//                         if !www_available().await {
//                             self.state = NetworkState::Failure {
//                                 at: Instant::now(),
//                                 next_check: (Instant::now(), Duration::from_secs(10))
//                             };
//                         }
//                     }

//                     if self.samples.len() >= SAMPLE_SIZE-1 {
//                         self.samples.pop();
//                     }

//                     self.samples.push(sample);
//                 }
//             }

//             NetworkState::TargetBlocking => {
//                 return Ok(SignalControl::Drop)
//             }

//             _ => {}
//         }

//         if self.sample_ctr >= SAMPLE_EVERY {
//             if let NetworkState::Ok = self.state {                
//                 self.state = NetworkState::Sampling(
//                     JobPoolSample::new(
//                         SAMPLE_SIZE,
//                         (0.05, 0.95),
//                         (meta.ttl/12, meta.ttl-(meta.ttl / 12)),
//                 ));
//             }
//             self.sample_ctr = 0;
//         }
//         else {
//             self.sample_ctr += 1;
//         }
        
//         Ok(*signal)
//     }

// }

fn check_sample(sample: &JobPoolSample) -> IndicatorFlags { 
    IndicatorFlags {
        fishy_positive_rate: sample.fail_bounds.1 <= sample.percentage(),
        fishy_negative_rate: sample.percentage() <= sample.fail_bounds.0,
        fishy_low_lifetime:  sample.avg_time() <= sample.duration_bounds.0,
        fishy_high_lifetime: sample.duration_bounds.1 <= sample.avg_time() 
    }
}

struct IndicatorFlags {
    fishy_positive_rate: bool,
    fishy_negative_rate: bool,
    fishy_low_lifetime: bool,
    fishy_high_lifetime: bool
}
