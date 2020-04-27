use tokio::{
    sync::mpsc,
    time::timeout
};

use super::core::{ScheduleExecutor, CRON, PostProcessor};

pub struct Schedule<Job, Ret>
{
    val_tx: mpsc::Sender<Ret>,
    commands: Vec<Job>,
}


impl<T: Clone, R> Schedule<T, R> {
    fn append(&mut self, jobs: &[T]) {
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

