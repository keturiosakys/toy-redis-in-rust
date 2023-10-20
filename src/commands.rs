use crate::resp::RespToken;

pub fn run_commands<'a>(parsed: RespToken) -> Result<Vec<u8>, anyhow::Error> {
    if let RespToken::Array((parsed, _)) = parsed {
        let command = &parsed[0];
        let args = &parsed[1..];

        if args.len() > 0 {
            println!("args: {}", &args[0]);
        }

        match command {
            RespToken::SimpleString(command) => {
                let string = command.to_ascii_lowercase();

                if string == b"ping" {
                    return Ok(b"+PONG\r\n".to_vec());
                } else {
                    return Ok(b"-ERR unknown command\r\n".to_vec());
                }
            }
            RespToken::BulkString((command, _)) => {
                let command = command.to_ascii_lowercase();

                if command == b"echo" {
                    if let RespToken::BulkString((message, length)) = &args[0] {
                        let length = format!("{}", length);
                        let length = length.as_bytes();
                        let mut response = [&[b'$'], length, &[b'\r', b'\n']].concat();
                        response = [response, message.to_vec(), b"\r\n".to_vec()].concat();
                        return Ok(response);
                    } else {
                        return Ok(b"-ERR wrong format\r\n".to_vec());
                    }
                } else if command == b"ping" {
                    return Ok(b"+PONG\r\n".to_vec());
                } else {
                    return Ok(b"-ERR unknown command\r\n".to_vec());
                }
            }
            _ => todo!(),
        }
    } else {
        todo!()
    }
}
