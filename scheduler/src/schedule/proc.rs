use super::{CRON, Consumer};
use std::future::Future;

use crate::error::Error;

use tokio::{
    sync::mpsc,
    time::{timeout, Instant, Duration, DelayQueue, Error as TimeError},
    task::JoinHandle,
    stream::StreamExt
};

use std::{
    pin::Pin,
    task::{Context, Poll},
    collections::HashMap,
};

use smallvec::SmallVec;


/// Runs Post Processors against `receiver<R>`, Executed in order of the `post_hooks` (0..)
/// ```txt
/// -------------------------------------
///                    1st          2nd
///   post_hooks = [ processor -> processor ] -> Some(Released)
///                             ^ or drop
/// -------------------------------------
/// ```
pub struct Handles<J>(Vec<JoinHandle<Option<J>>>);

enum HandlesState<J> {
    Push(J),
    Pop,
    Pending,
    Error(Box<dyn std::error::Error>)
}

impl<J> Handles<J> {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn push(&mut self, item: JoinHandle<Option<J>>) {
        self.0.push(item)
    }

    /// Like `join` except non-blocking
    /// Seeks for Resolved threads - remove resolved, keep unresolved until next pass
    pub async fn partial_join(&mut self, job_buf: &mut Vec<J>) {
        let mut i: usize = 0;
        

        let mut indexes: SmallVec<[usize; 256]> = SmallVec::new();
        

        for mut handle in &mut self.0 {
            let resp = futures::future::poll_fn(|cx: &mut Context| {
                match Pin::new(&mut handle).poll(cx) {
                    Poll::Ready(Ok(Some(job))) => Poll::Ready(HandlesState::Push(job)),
                    Poll::Ready(Ok(None)) => Poll::Ready(HandlesState::Pop),
                    Poll::Ready(Err(e)) => Poll::Ready(HandlesState::Error(Box::new(e))),
                    Poll::Pending => Poll::Ready(HandlesState::Pending)
                }
            }).await;
            
            match resp {
                HandlesState::Push(job) => {
                    job_buf.push(job);
                    indexes.push(i);
                },
                HandlesState::Pop => indexes.push(i),
                HandlesState::Pending => {},
                HandlesState::Error(e) => {
                    // trace::log!("shit {}", e);
                    indexes.push(i)
                }
            };
            i += 1;
        }

        remove_indexes(&mut self.0, &indexes[..]);
        //jobs
    }
}

fn remove_indexes<T>(src: &mut Vec<T>, indexes: &[usize]) {
    let mut balancer: usize = 0;
    for rm_i in indexes {
        let i = rm_i - balancer;
        balancer += 1;
        src.remove(i);
    }
}
