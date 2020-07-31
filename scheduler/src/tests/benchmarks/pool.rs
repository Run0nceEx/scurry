extern crate test;
use test::Bencher;

use crate::{
    schedule::{
        pool::CronPool,
        meta::CronMeta,
        SignalControl,
        CRON, Subscriber, MetaSubscriber
    },
    error::Error
};

use std::time::Duration;


#[bench]
fn noop(b: &mut Bencher) {
    use super::*;


}




#[bench]
fn rand_sleep(b: &mut Bencher) {
    use super::*;

}

#[bench]
fn const_sleep(b: &mut Bencher) {
    use super::*;

}



