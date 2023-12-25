use std::{
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::bail;

use crate::{
    ast::{
        Command::{Config, Echo, Get, Ping, Set},
        Pipeline,
    },
    models::Config as ServerConfig,
    resp::RespToken,
    ExpiringValue, RedisStore,
};

pub fn eval<'a>(
    pipeline: Pipeline,
    cache: Arc<Mutex<RedisStore>>,
    config: ServerConfig,
) -> Result<Vec<Vec<u8>>, anyhow::Error> {
    pipeline
        .commands
        .into_iter()
        .map(|cmd| match cmd {
            Ping => return RespToken::serialize(RespToken::SimpleString("PONG".as_bytes())),
            Echo(msg, len) => return RespToken::serialize(RespToken::BulkString((msg, len))),
            Set(key, val, duration) => {
                let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                let mut cache = cache.lock().unwrap();

                cache.insert(
                    key.to_vec(),
                    ExpiringValue {
                        value: val.to_vec(),
                        ts,
                        expiry: duration,
                    },
                );
                return RespToken::serialize(RespToken::SimpleString("OK".as_bytes()));
            }
            Get(key) => {
                let cache = cache.lock().unwrap();

                if let Some(ExpiringValue { value, ts, expiry }) = cache.get(&key) {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                    let expiry = expiry.unwrap_or(Duration::from_millis(0));
                    let lapsed = now.as_millis() - ts.as_millis();

                    if expiry.as_millis() > 0 && lapsed > expiry.as_millis() {
                        return RespToken::serialize(RespToken::Nil);
                    }

                    return RespToken::serialize(RespToken::BulkString((value, value.len() as u8)));
                } else {
                    return RespToken::serialize(RespToken::Nil);
                }
            }
            Config(cmd) => match cmd {
                crate::ast::ConfigCommands::Get(key) => match key.as_slice() {
                    b"dir" => {
                        let value = config.dir.as_bytes();
                        let response = vec![
                            RespToken::BulkString((&key, key.len() as u8)),
                            RespToken::BulkString((value, value.len() as u8)),
                        ];
                        return RespToken::serialize(RespToken::Array((
                            response.clone(),
                            response.len() as u8,
                        )));
                    }
                    b"dbfilename" => {
                        let value = config.db_file_name.as_bytes();
                        let response = vec![
                            RespToken::BulkString((&key, key.len() as u8)),
                            RespToken::BulkString((value, value.len() as u8)),
                        ];
                        return RespToken::serialize(RespToken::Array((
                            response.clone(),
                            response.len() as u8,
                        )));
                    }
                    _ => bail!("Unrecognized config key"),
                },
            },
        })
        .collect()
}
