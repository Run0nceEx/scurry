use std::{
    collections::HashMap,
    time::Duration,
};

use tokio_util::time::DelayQueue;

use tokio_stream::StreamExt;

pub struct Stash<T> {
    stash: HashMap<usize, T>,
    timer: DelayQueue<usize>,
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

        // todo(adam)
        // possible resource sink here
        let mut key: usize = rand::random();
        while let Some(_) = self.stash.get(&key) {
            key = rand::random();
        }
        // --

        let _key = self.timer.insert(key, *delay_for);
        self.stash.insert(key, state);

    }

    /// flushes all the states of tasks left inside of `Stash`
    /// and returns the number it placed at the tail of the buffer
    pub fn flush(&mut self, jobs: &mut Vec<T>) -> usize {
        let amount = self.stash.len();
        jobs.extend(
            self.stash
                .drain()
                .map(|(_id, state)| state)
        );
        
        self.timer.clear();
        amount
    }

    /// pushes states of tasks into `job` parameter and,
    /// returns the amount it placed at the tail of the buffer, 
    #[inline]
    pub async fn release(&mut self, jobs: &mut Vec<T>) -> usize {
        let mut amount = 0;
        while let Some(Ok(res)) = self.timer.next().await {
            if let Some(state) = self.stash.remove(res.get_ref()) {
                jobs.push(state);
                amount += 1;
            }
        }
        amount
    }

    #[inline]
    pub fn amount(&self) -> usize {
        self.timer.len()
    }
}
