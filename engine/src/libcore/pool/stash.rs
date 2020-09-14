use std::{
    collections::HashMap,
    time::Duration,
};

use tokio::{
    time::DelayQueue,
    stream::StreamExt
};

pub struct Stash<T> {
    stash: HashMap<usize, T>,
    timer: DelayQueue<usize>
}

impl<T> Stash<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            timer: DelayQueue::new(),
            stash: HashMap::new(),
        }
    }

    #[inline]
    pub fn insert(&mut self, state: T, delay_for: &Duration) {
        // ignoring key bc we dont transverse `self.stash` to remove items from
        // `self.timer`
        let mut key: usize = rand::random();
        while let Some(_) = self.stash.get(&key) {
            key = rand::random();
        }
        
        let _key = self.timer.insert(key, *delay_for);
        self.stash.insert(key, state);
    }

    /// Release tasks from Timer
    #[inline]
    pub async fn release(&mut self, jobs: &mut Vec<T>) {
        while let Some(Ok(res)) = self.timer.next().await {
            if let Some(state) = self.stash.remove(res.get_ref()) {
                jobs.push(state);
            }
        }
    }

    #[inline]
    pub fn amount(&self) -> usize {
        self.timer.len()
    }
}
