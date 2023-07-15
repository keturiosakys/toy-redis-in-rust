// Uncomment this block to pass the first stage
use std::{
    io::{Error, Write},
    net::{TcpListener, TcpStream},
};
use std::net;

fn main() -> Result<(), Error> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        let _ = handle_connection(stream);
    }

    Ok(())
}

fn handle_connection(stream: Result<TcpStream, Error>) -> Result<(), Error> {
    match stream {
        Ok(mut stream) => {
            println!("accepted new connection");
            stream.write("+PONG\r\n".as_bytes())?;
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
    Ok(())
}
