use tokio::{
    sync::mpsc,
    time::timeout
};
use std::{
    time::{Duration, Instant},
    net::SocketAddr,
};

use super::{ScheduleExecutor, CRON, PostProcessor, Scannable};
use serde::{Serialize, Deserialize};
use crate::utils::{DiskRepr, IntoDiskRepr, FromDiskRepr};


async fn spawn<T, R>(job: T, mut tx: mpsc::Sender<R>) where T: CRON<R> {
    match timeout(job.ttl(), job.exec()).await {
        Ok(val) => {
            if let Err(e) = tx.send(val).await {
                eprintln!("Failed to send back in Job: {}", e)
            }
        }
        Err(e) => {
            eprintln!("{}", e)
        }
    }
}


/// Runs Post Processors against `receiver<R>`, Executed in order of the `post_hooks` (0..~)
pub async fn post_process<R>(rx: &mut mpsc::Receiver<R>, post_hooks: &[R]) -> Option<R> 
where 
    R: PostProcessor<R> + Copy
{
    if let Some(mut item) = rx.recv().await {
        for hook in post_hooks {
            match hook.process(item).await {
                Some(new_item) => {
                    item = new_item;
                }
                None => return None
            }
        }
        return Some(item)
    }
    None
}

#[derive(Clone)]
pub struct Schedule<Job, Ret>
{
    val_tx: mpsc::Sender<Ret>,
    commands: Vec<Job>,
}


impl<T, R> Schedule<T, R> {
    fn append(&mut self, jobs: &[T]) where T: Clone {
        if jobs.len() > 0 {
            self.commands.extend(jobs.iter().map(|x| x.clone()))
        }
    }

    fn new() -> (Self, mpsc::Receiver<R>) {
        let (vtx, vrx) = mpsc::channel(1024);
        
        let schedule = Self {
            val_tx: vtx,
            commands: Vec::with_capacity(1024)
        };

        (schedule, vrx)
    }
}

impl<T, R> ScheduleExecutor for Schedule<T, R>
where
    R: Sync + Send + Copy + 'static,
    T: CRON<R> + Sync + Send + 'static + Clone,
{   
    /// Run tasks
    fn run(&mut self) {
        let commands = self.commands.clone();
        for (i, command) in commands.iter().enumerate() {
            if command.check() {       
                let vtx = self.val_tx.clone();
                let job = self.commands.remove(i);
                tokio::spawn(async move { spawn(job, vtx).await });
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JobsSerialized<T>(Vec<T>);
impl<T> DiskRepr for JobsSerialized<T> {}

/// Partially serialize into raw bytes, 
impl<'a, T, R> IntoDiskRepr<JobsSerialized<T>> for Schedule<T, R>
where T: Serialize + Deserialize<'a> {
    fn into_raw_repr(self) -> JobsSerialized<T> {
        JobsSerialized(self.commands)
    }
}

/// Partially Deserialization raw bytes into original
impl<'a, T, R> FromDiskRepr<'a, JobsSerialized<T>> for Schedule<T, R> where T: Deserialize<'a> {
    fn from_raw_repr(&mut self, buf: &'a [u8]) -> Result<(), Box<bincode::ErrorKind>> {
        self.commands = bincode::deserialize::<JobsSerialized<T>>(buf)?.0;
        Ok(())
    }
}

impl<'a, T: Serialize + Deserialize<'a>, R> DiskRepr for Schedule<T, R>
where T: IntoDiskRepr<T> + FromDiskRepr<'a, T> {}


#[derive(Clone)]
enum Error {}

const MAX_RESCHEDULE: usize = 4;

#[derive(Clone)]
enum Action {
    NoopConnection,
    ProtocolScan,
    ReschdeludeOnFail(Box<Action>, Duration,)
}

#[derive(Clone)]
struct Mined {
    ttl: Duration,
    ts_started: Instant,
    ts_stopped: Option<Instant>,
    addr: SocketAddr,
    result: Result<bool, Error>
}


struct MinerSchedule<T> {
    schedule: Schedule<T, Mined>
}

impl<T, C> MinerSchedule<T>
where T: CRON<Mined> + Scannable<C, Error> {
    fn add_job(&mut self, connector: C) {
        
    }
}