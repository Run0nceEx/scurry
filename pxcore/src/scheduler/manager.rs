use std::{
    time::{Instant, Duration},
};

use tokio::{
    sync::mpsc,
    time::timeout
};

use async_trait::async_trait;

/// Command run on (CRON)
#[async_trait]
pub trait CRON<R>: Sized {
    /// Run function, and then append to parent if more jobs are needed
    async fn exec(self) -> (R, Vec<Self>);

    /// check if command should be ran
    fn check(&self) -> bool;

    /// Times out task if result takes too long - default time is 1 minute
    fn timeout(&self) -> Duration {
        Duration::from_secs(60)
    }
}


pub struct Scheduler<T, R>
{
    val_tx: mpsc::Sender<R>,
    job_tx: mpsc::Sender<Vec<T>>,
    job_rx: mpsc::Receiver<Vec<T>>,
    commands: Vec<T>
}

impl<T, R> Scheduler<T, R>
where
    R: Sync + Send + Copy + 'static,
    T: CRON<R> + Sync + Send + 'static + Clone,
{   
    /// Run tasks
    pub fn run_tasks(&mut self) {
        let commands = self.commands.clone();
        
        let mut i = 0;
        for command in commands {
            if command.check() {       
                let mut vtx = self.val_tx.clone();
                let mut jtx = self.job_tx.clone();

                let job = self.commands.remove(i);
                tokio::spawn(async move {
                    if let Ok((v, jobs)) = timeout(job.timeout(), job.exec()).await {
                        
                        if let Err(e) = vtx.send(v).await {
                            panic!("Value TX: {}", e)
                        }

                        if let Err(e) = jtx.send(jobs).await {
                            panic!("Jobs TX: {}", e)
                        }

                    }
                });
            }
            i += 1;
        }
    }

    pub fn new() -> (Self, mpsc::Receiver<R>) {
        let (vtx, vrx) = mpsc::channel(1024);
        let (jtx, jrx) = mpsc::channel(1024);
        
        let schedule = Self {
            val_tx: vtx,
            job_tx: jtx,
            job_rx: jrx,
            commands: Vec::with_capacity(1024)
        };

        (schedule, vrx)
    }

    pub async fn join_cron(&mut self) {
        while let Some(jobs) = self.job_rx.recv().await {
            self.commands.extend(jobs);
        }
    }
}


// struct A {}

// #[async_trait]
// impl CRON<()> for A {
//     async fn exec(self) -> ((), Vec<Self>) {
//         ((), vec![])
//     }

//     fn check(&self) -> bool {
//         true
//     }
// }


// enum Schedules {
//     ExploreNet(Scheduler<A, ()>),
// }


