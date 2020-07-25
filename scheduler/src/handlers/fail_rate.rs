use crate::{
    schedule::{
        SignalControl,
        meta::CronMeta,
        pool::MetaSubscriber
    },
    error::Error,
};

use std::time::Duration;
use tokio::net::{UdpSocket, TcpStream};

#[derive(Debug, Clone)]
enum NetworkState {
    /// Everything seems to be working
    Ok,
    
    /// Assumes we can only contact talk locally
    LocalOnly,

    /// Assumes we can't reach anything 
    Failure,
    
    /// We've irritated the target, and we believe they've blocked us
    TargetBlocking,
    
    /// We believe that a failure has occured, 
    /// and this state will be temporarily kept
    /// until investigations are resolved 
    CheckingFailure {
        neg: usize,
        pos: usize,
        avg_duration: Duration
    }
}

struct Host;

pub async fn network() -> Option<Host> {

    let socket = UdpSocket::bind("127.0.0.1:34254").await.expect("couldn't bind to address");
    
    
    unimplemented!()
}

pub async fn local() -> Option<Host> {
    let socket = UdpSocket::bind("127.0.0.1:34254").await.expect("couldn't bind to address");
    unimplemented!()
}


#[derive(Debug, Clone)]
pub struct FailRateMonitor {
    threshold_percentage: f32,
    threshold_limiter: usize,       // Amount to buffer before precentages
    pub positive_cnt: usize,
    pub negative_cnt: usize,
    state: NetworkState,            // 
    
}

// a simple logger to check if a schedule is failing consistently
impl FailRateMonitor {
    pub fn new(threshold: (f32, usize)) -> Self {
        Self {
            threshold_percentage: threshold.0,
            threshold_limiter: threshold.1,
            positive_cnt: 0,
            negative_cnt: 0,
            state: NetworkState::Ok
        }
    }

    pub fn percentage(&self) -> f32 {
        ((self.positive_cnt+self.negative_cnt) as f32 * 0.01) / self.negative_cnt as f32 * 0.01
    }

    pub fn network_state(&self) -> &NetworkState {
        &self.state
    }
}

#[async_trait::async_trait]
impl MetaSubscriber for FailRateMonitor {
    async fn handle(&mut self, meta: &mut CronMeta, signal: &mut SignalControl) -> Result<(), Error> {
        match signal {
            SignalControl::Success => self.positive_cnt += 1,
            SignalControl::Drop | SignalControl::Fuck => self.negative_cnt += 1,
            _ => {}
        }
        
        // if self.network_failure {
        //     *signal = SignalControl::Reschedule(Duration::from_secs(360));
        // }

        if self.positive_cnt+self.negative_cnt >= self.threshold_limiter {
            let percentage = self.percentage();
            if percentage >= self.threshold_percentage {
                println!("{} is failing at {} rate", meta.handler_name, percentage)
            }
        }
        Ok(())
    }
}
