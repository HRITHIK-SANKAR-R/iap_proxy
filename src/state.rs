use std::sync::atomic::{AtomicU64,Ordering};
use std::net::IpAddr;
use dashmap::DashMap;

pub struct ProxyState{
    pub total:AtomicU64,
    pub blocked:AtomicU64,
    pub offenders:DashMap<IpAddr,u64>,
    pub target_addr:String,
}

impl ProxyState{
    pub fn new(target_addr:String)->ProxyState{
        ProxyState{
            total:AtomicU64::new(0),
            blocked:AtomicU64::new(0),
            offenders:DashMap::new(),
            target_addr,
        }
    }
}