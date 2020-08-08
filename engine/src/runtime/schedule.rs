// this will be the scheduler
// needs to be seperated from pool



pub struct Schedule<T>
{
    timer: DelayQueue<uuid::Uuid>,                     // timer for jobs
    pub bank: HashMap<uuid::Uuid, (CronMeta, T)>,      // collection of pending jobs
}


impl<T> Schedule<T>
{
    pub fn insert(&mut self, meta: CronMeta, state: T) {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, meta.tts);
        self.bank.insert(meta.id, (meta, state));
        
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            bank: HashMap::new(),
            timer: DelayQueue::new(),
        }
    }

    /// Release tasks from Timer
    /// If `max` is 0, no limit is occured
    pub async fn release_ready(&mut self, reschedule_jobs: &mut Vec<(CronMeta, S)>) -> Result<(), Error> 
    {
        // wait wtf timer is a stream?!
        while let Some(res) = self.timer.next().await {
            if let Some((meta, state)) = self.bank.remove(res?.get_ref()) {
                reschedule_jobs.push((meta, state));
            }
        }
        Ok(())
    }
}