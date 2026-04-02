mod relay;
mod identity;
mod state;

use std::env;
use tokio::net::{TcpListener};
use tracing::{info,error};
use std::sync::Arc;
use sqlx::PgPool;
use dashmap::DashMap;
use std::net::IpAddr;
use std::sync::atomic::{Ordering};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let proxy_address=env::var("PROXY_ADDRESS").unwrap_or_else(|_|"127.0.0.1:6969".to_string());

    let target_address = env::var("TARGET_ADDRESS").unwrap_or_else(|_| "127.0.0.1:6767".to_string());
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-immediately".to_string());

    // Initialize state with the secret
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // 2. Load existing offenders from DB into RAM (The DashMap)
    let saved_offenders = sqlx::query!("SELECT ip_address, strikes FROM offenders")
        .fetch_all(&pool)
        .await?;

    let offenders_map = DashMap::new();
    for row in saved_offenders {
        if let Ok(ip) = row.ip_address.parse::<IpAddr>() {
            offenders_map.insert(ip, row.strikes.unwrap_or(0) as u64);
        }
    }

    // Use the custom function to build the state
    let mut app_state = state::ProxyState::new(target_address, secret, pool);

    // Inject the pre-loaded hackers from the database
    app_state.offenders = offenders_map;

    // Wrap it in the thread-safe Arc
    let state = Arc::new(app_state);

    let listener=match TcpListener::bind(&proxy_address).await{
      Ok(listener)=>listener,
        Err(e)=> {
          error!("Error binding to address:{}",e);
          std::process::exit(1);
        }
    };

    info!("IAP Proxy Listening on {}",&proxy_address);

    loop{
        let (socket,addr)=match listener.accept().await{
            Ok((s,addr))=>(s,addr),
            Err(e)=> {
                error!("Error accepting connection:{}",e);
                continue;
            }
        };
        let client_ip=addr.ip();

        let task_state=Arc::clone(&state);
        task_state.total.fetch_add(1,Ordering::SeqCst);
        tokio::spawn(async move {
            if let Err(e) = relay::proxy_bridge(socket,task_state,client_ip).await {
                error!("Error in proxy bridge: {}", e);
            }
        });
    }
}