
use std::net::{TcpStream};
use std::io::{copy, Read, Result, Write};
use std::thread;
use crate::identity;

const MAX_HEADER_SIZE: usize = 16384; // 16 KB
pub fn proxy_bridge(mut client_stream:TcpStream, target_addr:&str,)->Result<()>{

    let mut buffer=vec![0u8;MAX_HEADER_SIZE];
    let n=match client_stream.read(&mut buffer){
        Ok(n)=>n,
        Err(e)=>return Err(e),
    };
    let first_chunk=&buffer[..n];

    // We need a logic to intercept and validate the headers.
    println!(">>> INTERCEPTED HEADERS ({} bytes):", n);
    println!("{}", String::from_utf8_lossy(first_chunk));
    if !identity::is_authorized(first_chunk){
        // We send a professional HTTP 401 response back to the user
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

        //Create a new TCP connection from server to client

        //Create a new TCP connection from client to server
        let _ = client_stream.write_all(response.as_bytes());
        // Returning early so that the connection never reaches the target server.
        return Ok(());
    }
    //
    let mut server_stream=match TcpStream::connect(target_addr){
        Ok(stream)=>stream,
        Err(e)=>return Err(e),
    };
    let _=match server_stream.write_all(first_chunk){
        Ok(_)=>{},
        Err(e)=>return Err(e),
    };

    let mut client_reader=match client_stream.try_clone(){
        Ok(stream)=>stream,
        Err(e)=>return Err(e),
    };

    let mut server_reader=match server_stream.try_clone(){
        Ok(stream)=>stream,
        Err(e)=>return Err(e),
    };

    // Pipe A: Client -> Proxy -> Server

    let client_to_server=thread::spawn(move ||{
        let _=copy(&mut client_reader,&mut server_stream);
    });

    // Pipe B: Server -> Proxy -> Client

    let _=copy(&mut server_reader,&mut client_stream);
    let _=client_to_server.join();

    Ok(())
}