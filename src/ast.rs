use std::{time::Duration, u128};

use anyhow::bail;

use crate::resp::RespToken;

pub enum Command<'a> {
    Ping,
    Echo(&'a [u8], u8),
    // key, val, duration
    Set(Vec<u8>, Vec<u8>, Option<Duration>),
    // key
    Get(Vec<u8>),
    Config(ConfigCommands),
}

pub enum ConfigCommands {
    Get(Vec<u8>),
}

pub struct Pipeline<'a> {
    pub commands: Vec<Command<'a>>,
}

impl<'a> Pipeline<'a> {
    pub fn new() -> Self {
        Pipeline {
            commands: Vec::new(),
        }
    }

    pub fn add_command(&mut self, command: Command<'a>) {
        self.commands.push(command);
    }
}

pub fn build_pipeline<'a>(lexed: RespToken<'a>) -> Result<Pipeline<'a>, anyhow::Error> {
    let mut pipeline = Pipeline::new();

    if let RespToken::Array((arr, _len)) = lexed {
        match arr[0] {
            RespToken::SimpleString(str) => {
                if let b"PING" = str.to_ascii_uppercase().as_slice() {
                    pipeline.add_command(Command::Ping);
                    Ok(pipeline)
                } else {
                    bail!("Unsupported command")
                }
            }
            RespToken::BulkString((bulk, _len)) => {
                match bulk.to_ascii_uppercase().as_slice() {
                    b"PING" => pipeline.add_command(Command::Ping),
                    b"ECHO" => {
                        if let Some(RespToken::BulkString((message, len))) = arr.get(1) {
                            pipeline.add_command(Command::Echo(message, *len))
                        } else {
                            bail!("Invalid Echo format")
                        }
                    }
                    b"SET" => {
                        let cmd = match (arr.get(1).cloned(), arr.get(2).cloned(), arr.get(3)) {
                            (Some(key), Some(value), Some(exp))
                                if exp.unpack().to_ascii_uppercase().as_slice() == b"PX" =>
                            {
                                let input_millis: u128 = std::str::from_utf8(
                                    arr.get(4).expect("Invalid PX command").unpack(),
                                )?
                                .parse()?;

                                let expiry = Duration::from_millis(input_millis.try_into()?);

                                Command::Set(
                                    key.unpack().to_vec(),
                                    value.unpack().to_vec(),
                                    Some(expiry),
                                )
                            }
                            (Some(key), Some(value), None) => {
                                Command::Set(key.unpack().to_vec(), value.unpack().to_vec(), None)
                            }
                            _ => bail!("Invalid SET format"),
                        };

                        pipeline.add_command(cmd);
                    }
                    b"GET" => {
                        let cmd = match arr.get(1).cloned() {
                            Some(val) => Command::Get(val.unpack().to_vec()),
                            None => {
                                bail!("No key provided GET")
                            }
                        };

                        pipeline.add_command(cmd)
                    }
                    b"CONFIG" => {
                        let cmd = match arr.get(2).cloned() {
                            Some(val) => {
                                Command::Config(ConfigCommands::Get(val.unpack().to_vec()))
                            }
                            None => bail!("No config key provided for CONFIG GET"),
                        };

                        pipeline.add_command(cmd)
                    }
                    _ => {
                        bail!("Unrecognized format")
                    }
                };

                Ok(pipeline)
            }
            RespToken::Array(_) => bail!("Nested arrays are not allowed in RESP"),
            RespToken::Nil => bail!("Invalid value"),
            RespToken::Error(_) => bail!("Invalid RESP value"),
        }
    } else {
        bail!("Commands can only be built from arrays");
    }
}
