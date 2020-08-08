use crate::{
    schedule::{
        SignalControl,
        meta::CronMeta,
    },
};

pub struct LatencyCheck;


#[async_trait::async_trait]
impl MetaSubscriber for LatencyCheck {
    async fn handle(&self, data: &mut CronMeta, signal: &mut SignalControl) -> Result<(), Error> {
        println!("{:?}", data)
    }
}

impl std::fmt::Display for PrintSub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "PrintSub")
    }
}

