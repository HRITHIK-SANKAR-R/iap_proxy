use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

pub async fn send_error(stream: &mut TcpStream, status: &str, body: &str) -> tokio::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status, body.len(), body
    );
    stream.write_all(response.as_bytes()).await
}