use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{resp::RespToken, RedisStore};

pub fn evaluate<'a>(
    parsed: RespToken,
    cache: Arc<Mutex<RedisStore>>,
) -> Result<Vec<u8>, anyhow::Error> {
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
                let command = command.to_ascii_uppercase();

                match command.as_slice() {
                    b"PING" => return Ok(b"+PONG\r\n".to_vec()),
                    b"ECHO" => {
                        if let RespToken::BulkString((message, length)) = &args[0] {
                            let length = format!("{}", length);
                            let length = length.as_bytes();
                            let mut response = [&[b'$'], length, &[b'\r', b'\n']].concat();
                            response = [response, message.to_vec(), b"\r\n".to_vec()].concat();
                            return Ok(response);
                        } else {
                            return Ok(b"-ERR wrong format\r\n".to_vec());
                        }
                    }
                    b"SET" => {
                        let key = args[0].unpack();
                        let value = args[1].unpack();
                        let expiry: Option<u128>;

                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis();

                        if let Some(val) = args.get(2) {
                            let val = val.unpack().to_ascii_uppercase();

                            if val == b"PX" {
                                let recorded_expiry: u128 = std::str::from_utf8(
                                    args.get(3).expect("Invalid expiry request").unpack(),
                                )?
                                .parse()?;

                                expiry = Some(recorded_expiry);
                            } else {
                                return Ok(b"-ERR unrecognized command\r\n".to_vec());
                            }
                        } else {
                            expiry = None;
                        }

                        let mut cache = cache.lock().unwrap();
                        cache.insert(
                            key.to_vec(),
                            (value.to_vec(), timestamp, expiry.unwrap_or(0)),
                        );
                        return Ok(b"+OK\r\n".to_vec());
                    }
                    b"GET" => {
                        let key = args[0].unpack();

                        let cache = cache.lock().unwrap();

                        if let Some((value, recorded_ts, expiry)) = cache.get(key) {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_millis();

                            let lapsed = now - recorded_ts;

                            if *expiry > 0 && lapsed > *expiry {
                                return Ok(b"$-1\r\n".to_vec());
                            }

                            let length = format!("{}", value.len());
                            let length = length.as_bytes();
                            let mut response = [&[b'$'], length, &[b'\r', b'\n']].concat();
                            response = [response, value.to_vec(), b"\r\n".to_vec()].concat();

                            return Ok(response);
                        } else {
                            return Ok(b"$-1\r\n".to_vec());
                        }
                    }
                    _ => return Ok(b"-ERR unknown command\r\n".to_vec()),
                }
            }
            _ => todo!(),
        }
    } else {
        todo!()
    }
}
