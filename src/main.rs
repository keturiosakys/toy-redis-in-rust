use std::{
    io::{Error, Read, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<(), Error> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379")?;

    for stream in listener.incoming() {
        let _ = handle_connection(stream?);
    }

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), anyhow::Error> {
    println!("Incoming connection from: {}", stream.peer_addr()?);

    let mut read_buffer = [0; 2048];
    loop {
        match stream.read(&mut read_buffer) {
            Ok(received_size) => {
                if received_size == 0 {
                    println!("Connection closed");
                    return Ok(());
                }

                let received = &read_buffer[..received_size];
                println!("Got: {}", String::from_utf8_lossy(received));
                let response = "+PONG\r\n";

                stream.write(response.as_bytes())?;
                stream.flush()?;
            }
            Err(error) => eprintln!("Error reading from stream: {}", error),
        };
    }
}
