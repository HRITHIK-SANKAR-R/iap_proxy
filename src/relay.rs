mod router;
mod responses;

use std::net::IpAddr;
use tokio::net::TcpStream;
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt, Result};
use tracing::{info, error, warn};
use std::sync::{Arc, atomic::Ordering};
use crate::state::ProxyState;
use crate::identity;

const MAX_HEADER_SIZE: usize = 16384;

pub async fn proxy_bridge(mut client_stream: TcpStream, state: Arc<ProxyState>, client_ip: IpAddr) -> Result<()> {
    // 1. IP Check
    if let Some(fail_cnt) = state.offenders.get(&client_ip) {
        if *fail_cnt >= 3 {
            responses::send_error(&mut client_stream, "403 Forbidden", "Access Denied").await?;
            return Ok(());
        }
    }

    // 2. Read Request
    let mut buffer = vec![0u8; MAX_HEADER_SIZE];
    let n = client_stream.read(&mut buffer).await?;
    if n == 0 { return Ok(()); }
    let first_chunk = &buffer[..n];

    // 3. Auth & Routing
    let claims = match identity::is_authorized(first_chunk, &state.decoding_key) {
        Some(c) => c,
        None => {
            let mut entry = state.offenders.entry(client_ip).or_insert(0);
            *entry += 1;
            responses::send_error(&mut client_stream, "401 Unauthorized", "Valid Token Required").await?;
            return Ok(());
        }
    };

    let target = match router::get_route(first_chunk, &claims, &state.target_addr) {
        Ok(t) => t,
        Err(e) => {
            responses::send_error(&mut client_stream, "403 Forbidden", &e).await?;
            return Ok(());
        }
    };

    // 4. Connect & Bridge
    info!("🚀 Routing: {} -> {}", client_ip, target);
    let mut server_stream = match TcpStream::connect(&target).await {
        Ok(s) => s,
        Err(_) => {
            responses::send_error(&mut client_stream, "502 Bad Gateway", "Upstream Offline").await?;
            return Ok(());
        }
    };

    server_stream.write_all(first_chunk).await?;
    let (mut cr, mut cw) = client_stream.into_split();
    let (mut sr, mut sw) = server_stream.into_split();

    let c2s = tokio::spawn(async move { let _ = copy(&mut cr, &mut sw).await; });
    let _ = copy(&mut sr, &mut cw).await;
    let _ = c2s.await;

    Ok(())
}