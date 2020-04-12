use std::{
    time::{Instant, Duration},
    collections::HashMap,
};


use tokio::{
    task::JoinHandle,
    sync::mpsc,
    time::timeout
};


use async_trait::async_trait;


// Command run on (CRON)
#[async_trait]
pub trait CRON<R>: Sized {
    /// Run function, and then append to parent if more jobs are needed
    async fn exec(self) -> (R, Vec<Self>);

    /// check if command should be ran
    fn check(&self) -> bool;


    fn timeout_ms(&self) -> u64 {
        3000
    }
}


pub struct Scheduler<T, R>
{
    val_tx: mpsc::Sender<R>,
    commands: HashMap<uuid::Uuid, T>,
}

impl<T, R> Scheduler<T, R>
where
    R: Sync + Send + Copy + 'static,
    T: CRON<R> + Sync + Send + 'static + Clone,
{
    fn run_tasks(&mut self) -> Vec<JoinHandle<Vec<T>>> {
        let mut ret = Vec::new();
        let commands = self.commands.clone();

        for (id, command) in commands {
            if command.check() {       
                let mut vtx = self.val_tx.clone();

                match self.commands.remove(&id) {
                    Some(job) => 
                    {
                        ret.push(
                            tokio::spawn(async move {
                                if let Ok((v, jobs)) = timeout(Duration::from_secs(job.timeout_ms()), job.exec()).await {
                                    vtx.send(v).await;
                                    return jobs
                                }
                                vec![]
                            }
                        ));
                    }
                    None => {} // Add error handle?
                }

            }
        }

        return ret;
    }

    fn new() -> (Self, mpsc::Receiver<R>) {
        unimplemented!()
    }
}

#[async_trait]
trait ScheduleJoin<T>: Sync + Send
where 
    T: 'static + Send + Sync
{
    fn get_handles(&self) -> Vec<JoinHandle<Vec<T>>>;

    async fn join_handles(&mut self) -> Vec<T> {
        let mut ret: Vec<T> = Vec::new();
        for x in self.get_handles() {
            if let Ok(jobs) = x.await {
                ret.extend(jobs);
            }
        }
        ret
    }
}


struct A {}

#[async_trait]
impl CRON<()> for A {
    async fn exec(self) -> ((), Vec<Self>) {
        ((), vec![])
    }

    fn check(&self) -> bool {
        true
    }
}


enum Schedules {
    ExploreNet(Scheduler<A, ()>),

}


// struct Manager {
//     schedules: Vec<>
// }



// trait GenericConsumer {
    
// }
// trait GenericSchedule {
//     fn emit_ready<Arg, R>(&mut self) -> Vec<JobSchedule<Arg, R>>;
// }


// struct Manager<T: GenericSchedule> {
//     schedules: Vec<T>

// }

