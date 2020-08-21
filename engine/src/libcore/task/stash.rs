use std::{
    collections::HashMap,
    time::Duration,
};

use super::{
    meta::CronMeta
};

use tokio::{
    time::DelayQueue,
    stream::StreamExt
};

pub struct Stash<T> {
    stash: HashMap<uuid::Uuid, (CronMeta, T)>,
    timer: DelayQueue<uuid::Uuid>
}


impl<T> Stash<T> 
{
    #[inline]
    pub fn new() -> Self {
        Self {
            timer: DelayQueue::new(),
            stash: HashMap::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, meta: CronMeta, state: T, delay_for: &Duration) {
        // ignoring key bc we dont transverse `self.pending` to remove items from
        // `self.timer`
        let _key = self.timer.insert(meta.id, *delay_for);
        self.stash.insert(meta.id, (meta, state));
    }

    /// Release tasks from Timer
    #[inline]
    pub async fn release(&mut self, jobs: &mut Vec<(CronMeta, T)>) {
        while let Some(Ok(res)) = self.timer.next().await {
            if let Some((meta, state)) = self.stash.remove(res.get_ref()) {
                jobs.push((meta, state));
            }
        }
    }

    pub fn amount(&self) -> usize {
        self.timer.len()
    }
}
