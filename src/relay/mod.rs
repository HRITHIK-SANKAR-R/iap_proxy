pub mod router;
pub mod responses;

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::net::IpAddr;
use tokio::net::TcpStream;
use tracing::{info, warn, error};

use crate::state::ProxyState;
use crate::identity;

pub async fn proxy_bridge(
    mut client_stream: TcpStream,
    state: Arc<ProxyState>,
    client_ip: IpAddr,
) -> Result<(), Box<dyn std::error::Error>> {

    // 1. Read the raw bytes from the network WITHOUT consuming them
    let mut buffer = [0; 4096];
    let bytes_peeked = match client_stream.peek(&mut buffer).await {
        Ok(n) if n == 0 => return Ok(()), // Connection closed immediately
        Ok(n) => n,
        Err(e) => {
            error!("Failed to peek at stream: {}", e);
            return Ok(());
        }
    };

    // 2. Check identity using the bytes we just peeked at (Notice: NO .await)
    let auth_result = identity::is_authorized(&buffer[..bytes_peeked], &state.decoding_key);

    match auth_result {        // --- THE HACKER PATH ---
        None => {
            // Update RAM
            state.blocked.fetch_add(1, Ordering::SeqCst);
            let mut entry = state.offenders.entry(client_ip).or_insert(0);
            *entry += 1;
            let current_strikes = *entry as i32;

            warn!("Auth Fail: {} | Strike: {}/3", client_ip, current_strikes);

            // Update Database (Background Task)
            let pool = state.db.clone();
            let ip_str = client_ip.to_string();

            tokio::spawn(async move {
                let res = sqlx::query!(
                    "INSERT INTO offenders (ip_address, strikes)
                     VALUES ($1, $2)
                     ON CONFLICT (ip_address)
                     DO UPDATE SET strikes = $2, last_attack = CURRENT_TIMESTAMP",
                    ip_str,
                    current_strikes
                )
                    .execute(&pool)
                    .await;

                if let Err(e) = res {
                    error!("DB Sync Error for {}: {}", ip_str, e);
                }
            });

            // Kick them out
            responses::send_error(&mut client_stream, "401 Unauthorized", "Valid Token Required").await?;
            return Ok(());
        }

        // --- THE VIP PATH ---
        Some(claims) => {
            info!("✅ Auth Success: {} (Role: {})", claims.sub, claims.role);

            // 1. NEW: Ask the Router where this traffic should go
            let target_url = match router::get_route(&buffer[..bytes_peeked], &claims, &state.target_addr) {
                Ok(t) => t,
                Err(e) => {
                    warn!("🚫 RBAC Denied for {}: {}", claims.sub, e);
                    responses::send_error(&mut client_stream, "403 Forbidden", "Access Denied: Admins Only").await?;
                    return Ok(());
                }
            };

            // 2. Connect to the dynamically routed target (Notice we use 'target_url' now)
            let mut target_stream = match TcpStream::connect(&target_url).await {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to connect to target {}: {}", target_url, e);
                    responses::send_error(&mut client_stream, "502 Bad Gateway", "Target Offline").await?;
                    return Ok(());
                }
            };

            // 3. Stream data back and forth infinitely
            match tokio::io::copy_bidirectional(&mut client_stream, &mut target_stream).await {
                Ok((from_client, from_server)) => {
                    info!("🔌 Connection Closed. Bytes sent: {}, Bytes received: {}", from_client, from_server);
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                }
            }
        }
    }

    Ok(())
}