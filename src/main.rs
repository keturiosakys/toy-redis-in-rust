mod ast;
mod commands;
mod models;
mod resp;
mod utils;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Error;
use models::{Config, ExpiringValue};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use clap::Parser;

use crate::resp::RespToken;

type RedisStore = HashMap<Vec<u8>, ExpiringValue>;

#[derive(Parser)]
#[clap(author,version, about, long_about = None)]
struct Opts {
    #[arg(long)]
    dir: String,
    #[arg(long = "dbfilename")]
    db_file_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let opts: Opts = Opts::parse();

    let config = Config {
        dir: opts.dir,
        db_file_name: opts.db_file_name,
    };

    let cache = Arc::new(Mutex::new(RedisStore::new()));

    loop {
        let (stream, _) = listener.accept().await?;
        let cache = cache.clone();
        let config = config.clone();
        tokio::spawn(async move {
            handle_connection(stream, cache, config)
                .await
                .map_err(|e| eprintln!("{:?}", e))
        });
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    cache: Arc<Mutex<RedisStore>>,
    config: Config,
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

                let pipeline = ast::build_pipeline(parsed)?;

                let response = commands::eval(pipeline, cache.clone(), config.clone())?;

                for bts in response {
                    stream.write(&bts).await?;
                }

                stream.flush().await?;
            }
            Err(error) => eprintln!("Error reading from stream: {}", error),
        };
    }
}
