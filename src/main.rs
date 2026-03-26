mod relay;
mod identity;
mod state;

use std::env;
use tokio::net::{TcpListener};
use tracing::{info,error};
use std::sync::Arc;
use std::sync::atomic::{Ordering};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let proxy_address=env::var("PROXY_ADDRESS").unwrap_or_else(|_|"127.0.0.1:6969".to_string());

    let target_address = env::var("TARGET_ADDRESS").unwrap_or_else(|_| "127.0.0.1:6767".to_string());
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "change-me-immediately".to_string());

    // Initialize state with the secret
    let state = Arc::new(state::ProxyState::new(target_address, secret));

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