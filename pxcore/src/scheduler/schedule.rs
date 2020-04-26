use tokio::{
    sync::mpsc,
    time::timeout
};
use super::core::{ScheduleExecutor, CRON};

pub struct Schedule<T, R>
{
    val_tx: mpsc::Sender<R>,
    commands: Vec<T>
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
                let mut vtx = self.val_tx.clone();
                let job = self.commands.remove(i);
                tokio::spawn(async move {
                    
                    if let Ok(val) = timeout(job.ttl(), job.exec()).await {
                        if let Err(e) = vtx.send(val).await {
                            panic!("Value TX: {}", e)
                        }
                    }
                
                });
            }
        }
    }
}