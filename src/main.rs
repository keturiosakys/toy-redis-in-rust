mod commands;
mod resp;
mod utils;

use anyhow::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::resp::RespToken;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(async move { handle_connection(stream).await });
    }
}

async fn handle_connection(mut stream: TcpStream) -> Result<(), anyhow::Error> {
    println!("Incoming connection from: {}", stream.peer_addr()?);

    let mut read_buffer = [0; 2048];
    loop {
        match stream.read(&mut read_buffer).await {
            Ok(received_size) => {
                if received_size == 0 {
                    println!("Connection closed");
                    return Ok(());
                }

                let received = &read_buffer[..received_size];
                println!("Got: {}", String::from_utf8_lossy(received));

                let parsed = RespToken::deserialize(received)?;

                let response = commands::run_commands(parsed)?;

                stream.write(&response).await?;
                stream.flush().await?;
            }
            Err(error) => eprintln!("Error reading from stream: {}", error),
        };
    }
}
