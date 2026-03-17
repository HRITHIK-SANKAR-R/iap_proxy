mod relay;
mod identity;
use tokio::net::{TcpListener};
use dotenvy::dotenv;
// use tracing_subscriber;
use tracing::{info,error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let proxy_address="127.0.0.1:6969";
    let target_address="127.0.0.1:6767";

    let listener=match TcpListener::bind(proxy_address).await{
      Ok(listener)=>listener,
        Err(e)=> {
          error!("Error binding to address:{}",e);
          std::process::exit(1);
        }
    };

    info!("IAP Proxy Listening on {}",proxy_address);

    loop{
        let socket=match listener.accept().await{
            Ok((s,_addr))=>s,
            Err(e)=> {
                error!("Error accepting connection:{}",e);
                continue;
            }
        };

        let target=target_address.to_string();

        tokio::spawn(async move {
            if let Err(e) = relay::proxy_bridge(socket,target).await {
                error!("Error in proxy bridge: {}", e);
            }
        });
    }
}