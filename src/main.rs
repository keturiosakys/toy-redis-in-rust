mod commands;
mod resp;
mod utils;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::resp::RespToken;

type RedisStore = HashMap<Vec<u8>, (Vec<u8>, u128, u128)>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let cache = Arc::new(Mutex::new(RedisStore::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        let cache = cache.clone();
        tokio::spawn(async move { handle_connection(stream, cache).await });
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    cache: Arc<Mutex<RedisStore>>,
) -> Result<(), anyhow::Error> {
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

                let response = commands::evaluate(parsed, cache.clone())?;

                stream.write(&response).await?;
                stream.flush().await?;
            }
            Err(error) => eprintln!("Error reading from stream: {}", error),
        };
    }
}
