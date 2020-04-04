use std::future::Future;
use std::time::Instant;

trait Task<Arg: Sized, R>: Fn(Arg) -> R + 'static
where R: Future<Output=R> {}


struct Job<Arg, R> {
    job: Box<dyn Task<Arg, R>>,
    queued_at: Instant,
    created_at: Instant
}


struct ProxyMine(Job<(), ()>);
struct ProxyDiscover(Job<(), ()>);


trait IntoJob<Arg, R> {
    fn into_task(self) -> Box<Task<Arg, R>>;
    fn as_job<'a>(&'a self) -> &'a Job<Arg, R>;
}


struct JobSchedule<Arg, R> {
    tasks: Vec<Box<IntoJob<Arg, R>>>
}


trait Scheduler {
    fn emit_ready<Arg, R>(&mut self) -> Vec<Box<IntoJob<Arg, R>>>;
}

struct Manager<T: Scheduler> {
    schedules: Vec<T>
}

