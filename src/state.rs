use std::sync::atomic::{AtomicU64};
use std::net::IpAddr;
use dashmap::DashMap;
use jsonwebtoken::DecodingKey;
use sqlx::PgPool;

pub struct ProxyState{
    pub total:AtomicU64,
    pub blocked:AtomicU64,
    pub offenders:DashMap<IpAddr,u64>,
    pub target_addr:String,
    pub decoding_key:DecodingKey,
    pub db:PgPool,
}

impl ProxyState{
    pub fn new(target_addr:String,secret:String,db:PgPool)->ProxyState{
        ProxyState{
            total:AtomicU64::new(0),
            blocked:AtomicU64::new(0),
            offenders:DashMap::new(),
            target_addr,
            decoding_key:DecodingKey::from_secret(secret.as_bytes()),
            db,
        }
    }
}