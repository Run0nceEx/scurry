mod core;
mod proc;

use async_trait::async_trait;
pub use crate::schedule::core::*;


#[async_trait]
pub trait Consumer<M: Clone + Send + 'static, T: Clone + Send + 'static> {
    async fn preprocess(&mut self, meta: M) -> bool {
        true
    }

    async fn postprocess(&mut self, result: T);
}

/// Used in scheduler (Command run on)
#[async_trait]
pub trait CRON<R>: Sized {
    /// Run function, and then append to parent if more jobs are needed
    async fn exec(&mut self) -> R;
}

