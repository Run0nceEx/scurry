struct State {
    display_timer: (Instant, f32),


    connect_pool: CronPool<OpenPortJob, PortState, Job>,
    job_stash: Stash<Job>,
    job_buf: Vec<(CronMeta, SignalControl, Option<PortState>, Job)>,

}
