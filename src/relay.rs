use std::net::{TcpStream,Shutdown};
use std::io::{copy,Result};
use std::thread;
pub fn proxy_bridge(mut client_stream:TcpStream, target_addr:&str,)->Result<()>{
    let mut server_stream=match TcpStream::connect(target_addr){
        Ok(stream)=>stream,
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