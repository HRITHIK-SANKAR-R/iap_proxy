use std::net::IpAddr;
use tokio::net::{TcpStream};
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt,Result};
use tracing::{info,error,warn,debug};
use crate::identity;
// use std::net::IpAddr;
use crate::state::ProxyState;
use std::sync::{Arc,atomic::{Ordering}};
const MAX_HEADER_SIZE: usize = 16384; // 16 KB
pub async fn proxy_bridge(mut client_stream:TcpStream,
                          state:Arc<ProxyState> ,
                          client_ip:IpAddr,
                          )->Result<()>{

    // 1st Che2cking of the Offending List or Blocked List to atop IP from accessing it
    if let(Some(fail_cnt))=state.offenders.get(&client_ip){
        if *fail_cnt>3{
            warn!("IP Blocked: {} - Strikes {}",client_ip,*fail_cnt);
            let res = "HTTP/1.1 403 Forbidden\r\nContent-Length: 15\r\n\r\nAccess Denied\n";
            match client_stream.write_all(res.as_bytes()).await{
                Ok(_)=>{},
                Err(e)=>return Err(e),
            };
            return Ok(());
        }
    }

    //2nd Getting the first chunk of the request

    let mut buffer=vec![0u8;MAX_HEADER_SIZE];
    let n=match client_stream.read(&mut buffer).await{
        Ok(n)=>n,
        Err(e)=>return Err(e),
    };
    let first_chunk=&buffer[..n];

    // We need a logic to intercept and validate the headers. We are checking the Authentication Token
    debug!(">>> INTERCEPTED HEADERS ({} bytes):", n);
    debug!("{}", String::from_utf8_lossy(first_chunk));
    if !identity::is_authorized(first_chunk,&state.decoding_key){
        state.blocked.fetch_add(1,Ordering::SeqCst);
        let mut entry=state.offenders.entry(client_ip).or_insert(0);
        *entry+=1;
        // We send a professional HTTP 401 response back to the user
        warn!("Unauthorized Access attempt blocked: {} There are {} more strikes left to get completely blocked", client_ip,4-(*entry));
        let body = "401 Unauthorized: Access Denied. Valid IAP Token Required.\n";
        let response = format!(
            "HTTP/1.1 401 Unauthorized\r\n\
             Content-Type: text/plain\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            body.len(),
            body
        );
        //Create a new TCP connection from client to server
        match client_stream.write_all(response.as_bytes()).await{
            Ok(_)=>{},
            Err(e)=>return Err(e),
        }
        // Returning early so that the connection never reaches the target server.
        return Ok(());
    }
    info!("Connection Authorized: Forwarding {} to {}", client_ip, &state.target_addr);
    //Create a new TCP connection from server to client
    let mut server_stream=match TcpStream::connect(&state.target_addr.to_string()).await{
        Ok(stream)=>stream,
        Err(e)=>{
            error!("Upstream Offline: {} -> {}", &state.target_addr, e);
            let body = "502 Bad Gateway: The backend server is unreachable.\n";
            let response = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Length: {}\r\n\r\n{}",
                body.len(), body
            );
            match client_stream.write_all(response.as_bytes()).await{
                Ok(_)=>{},
                Err(e)=>return Err(e),
            }
            return Ok(());
        }
    };
    info!("Connection Authorized: Forwarding to {}", &state.target_addr);
    match server_stream.write_all(first_chunk).await{
        Ok(_)=>{},
        Err(e)=>return Err(e),
    };

    let (mut client_reader,mut client_writer)= client_stream.into_split();
    let (mut server_reader,mut server_writer)= server_stream.into_split();


    // Pipe A: Client -> Proxy -> Server

    let client_to_server=tokio::spawn(async move {
        let _=copy(&mut client_reader,&mut server_writer).await;
    });

    // Pipe B: Server -> Proxy -> Client

    let _=copy(&mut server_reader,&mut client_writer).await;
    let _=client_to_server.await;

    Ok(())
}