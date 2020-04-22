
use super::manager::CRON;
use async_trait::async_trait;
use std::net::SocketAddr;

struct IPScan {
    addr: SocketAddr,
}


#[async_trait]
impl CRON<()> for IPScan {
    async fn exec(self) -> ((), Vec<Self>) {
        ((), vec![])
    }

    fn check(&self) -> bool {
        true
    }
}

