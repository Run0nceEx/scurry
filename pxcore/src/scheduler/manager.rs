use std::{
    time::{Instant, Duration},
    collections::HashMap,
};
use tokio::sync::mpsc;
use async_trait::async_trait;


#[derive(Copy, Clone)]
struct JobMeta {
    id: uuid::Uuid,
    exec_at: Instant,
    created_at: Instant
}

impl std::cmp::PartialEq for JobMeta {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}


#[async_trait]
trait Task<R>: Sized {
    // Run task, and reproduce more jobs if needed
    async fn exec(self) -> (R, Vec<(Self, JobMeta)>);
}


struct Scheduler<T, R>
{
    val_consumer: mpsc::Sender<R>,
    //job_consumer: mpsc::Sender<Vec<(T, JobMeta)>>,
    jobs: HashMap<uuid::Uuid, T>,
    schedule: HashMap<uuid::Uuid, JobMeta>
}


impl<T, R> Scheduler<T, R>
where
    T: Task<R> + Sync + Send + 'static,
    R: Sync + Send + Copy + 'static
{
    fn run_tasks(&mut self, job_tx: mpsc::Sender<Vec<(T, JobMeta)>> ) {
        let mut tasked = Vec::new(); 
        for (id, meta) in self.schedule.iter() {
            if Instant::now() - meta.exec_at > Duration::from_secs(0) {
                tasked.push(id.clone());
                
                let mut vtx = self.val_consumer.clone();
                let mut jtx = job_tx.clone();

                match self.jobs.remove(&id) {
                    Some(job) => {
                        tokio::spawn(async move {
                            // Fail handle?
                            let (v, jobs) = job.exec().await;
                            vtx.send(v).await;
                            jtx.send(jobs).await;
                        });

                    }
                    
                    None => {} // Add error handle?
                }
            }
        }

        for id in tasked {
            self.schedule.remove(&id);
        }

    }
}


struct A {

}

#[async_trait]
impl Task<()> for A {
    async fn exec(self) -> ((), Vec<(Self, JobMeta)>) {
        (
            (), vec![(
                A {}, JobMeta {
                    id: uuid::Uuid::default(),
                    created_at: Instant::now(),
                    exec_at: Instant::now()
                }
            )]
        )
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

