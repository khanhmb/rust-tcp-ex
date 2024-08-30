use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;
use tokio::time::sleep;

use libc::{c_uint, c_ulong, c_void};

#[no_mangle]
pub extern "C" fn getauxval(type_: c_uint)-> c_uint {
    0
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");

    // Spawn a task to run the client after a delay
    tokio::spawn(async {
        // Wait for 1 second before the client connects
        sleep(Duration::from_secs(1)).await;

        match TcpStream::connect("127.0.0.1:8080").await {
            Ok(mut stream) => {
                println!("Client connected to the server!");

                // Send a message to the server
                if let Err(e) = stream.write_all(b"hello").await {
                    eprintln!("Failed to write to server; err = {:?}", e);
                }

                let mut buf = [0; 1024];
                match stream.read(&mut buf).await {
                    Ok(n) if n > 0 => {
                        println!("Received from server: {}", String::from_utf8_lossy(&buf[..n]));
                    }
                    Ok(_) => {
                        println!("Connection closed by server");
                    }
                    Err(e) => {
                        eprintln!("Failed to read from server; err = {:?}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to server; err = {:?}", e);
            }
        }
    });

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n == 0 => {
                        println!("Connection closed by {}", addr);
                        return;
                    }
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("Failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                let received = String::from_utf8_lossy(&buf[0..n]);

                let response = if received.trim() == "hello" {
                    "world\n"
                } else {
                    "unrecognized command\n"
                };

                if let Err(e) = socket.write_all(response.as_bytes()).await {
                    eprintln!("Failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
