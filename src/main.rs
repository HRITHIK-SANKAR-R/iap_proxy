mod relay;
use std::net::{TcpListener};
use std::thread;
fn main() {
    let proxy_address="127.0.0.1:6969";
    let target_address="127.0.0.1:6767";

    let listener=match TcpListener::bind(proxy_address){
        Ok(listener)=>listener,
        Err(e)=>panic!("Failed to bind to address:{}",e),
    };

    println!("Listening on {}",proxy_address);

    for stream in listener.incoming(){
        match stream{
            Ok(stream)=>{
                println!("Accepted connection from {}",stream.peer_addr().unwrap());
                thread::spawn(move ||{
                    if let Err(e) = relay::proxy_bridge(stream,target_address){
                        eprintln!("Error proxying connection:{}",e);
                    }
                });
            }
            Err(e)=>{
                eprintln!("Error accepting connection:{}",e);
            }
        }
    }
}