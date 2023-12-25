use anyhow::{bail, Error};

pub const CR: u8 = b'\r';
pub const LF: u8 = b'\n';

#[derive(Debug, Clone)]
pub enum RespToken<'a> {
    Nil,
    #[allow(dead_code)]
    Error(RespError),
    SimpleString(&'a [u8]),
    BulkString((&'a [u8], u8)),
    Array((Vec<RespToken<'a>>, u8)),
}

#[derive(Debug, Clone)]
pub enum RespError {
    #[allow(dead_code)]
    InvalidCommand,
    #[allow(dead_code)]
    InvalidFormat,
}

impl<'a> RespToken<'a> {
    pub fn unpack(&self) -> &[u8] {
        match *self {
            RespToken::SimpleString(val) => val,
            RespToken::BulkString((val, _)) => val,
            _ => panic!("Can't unpack array"), // TODO: probably not a good interface
        }
    }

    pub fn deserialize(input: &'a [u8]) -> Result<Self, anyhow::Error> {
        let (res, rem) = parse_input(input)?;
        if rem.len() > 0 {
            bail!("Invalid input: {}", String::from_utf8_lossy(rem))
        }
        Ok(res)
    }

    pub fn serialize(token: RespToken) -> Result<Vec<u8>, anyhow::Error> {
        match token {
            RespToken::Nil => return Ok(b"$-1\r\n".to_vec()),
            RespToken::Error(err) => match err {
                RespError::InvalidCommand => return Ok(b"-ERR unknown command\r\n".to_vec()),
                RespError::InvalidFormat => return Ok(b"-ERR unknown format\r\n".to_vec()),
            },
            RespToken::SimpleString(str) => {
                let msg = [[b'+'].to_vec(), str.to_vec(), b"\r\n".to_vec()].concat();
                return Ok(msg);
            }
            RespToken::BulkString((bulk, len)) => {
                let len = format!("{}", len);
                let len = len.as_bytes();
                let mut res = [[b'$'].to_vec(), len.to_vec(), b"\r\n".to_vec()].concat();
                res = [res, bulk.to_vec(), b"\r\n".to_vec()].concat();
                return Ok(res);
            }
            RespToken::Array((arr, len)) => {
                let mut res = Vec::with_capacity(len.into());

                for token in arr {
                    let serialized = RespToken::serialize(token)?;
                    res.extend(serialized);
                }

                return Ok(res);
            }
        }
    }
}

pub fn parse_input(input: &[u8]) -> Result<(RespToken, &[u8]), anyhow::Error> {
    match input[0] {
        b'+' => parse_simple_string(&input[1..]),
        b'$' => parse_bulk_string(&input[1..]),
        b'*' => parse_array(&input[1..]),
        byte => bail!(
            "Invalid type specifier {}, must be valid RESP protocol",
            byte as char
        ),
    }
}

pub fn parse_until_eol(input: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    for (idx, el) in input.iter().enumerate() {
        if el == &CR {
            if input[idx + 1] != LF {
                bail!("Invalid EOL format")
            }
            return Ok((&input[..idx], &input[idx + 2..]));
        }
    }
    bail!("not enough bytes")
}

pub fn parse_array(input: &[u8]) -> Result<(RespToken, &[u8]), Error> {
    let (size, data) = parse_until_eol(input)?;
    let mut data = data;
    let size: usize = std::str::from_utf8(size)?.parse()?;

    let mut res = Vec::with_capacity(size);

    for _ in 0..size {
        let (resp, rest) = parse_input(data)?;
        res.push(resp);
        data = rest;
    }

    return Ok((RespToken::Array((res, size as u8)), data));
}

pub fn parse_simple_string<'a>(input: &'a [u8]) -> Result<(RespToken<'a>, &[u8]), Error> {
    if input.len() < 2 {
        bail!("Invalid simple string format")
    }

    let (data, rest) = parse_until_eol(input)?;
    return Ok((RespToken::SimpleString(data), rest));
}

pub fn parse_bulk_string(input: &[u8]) -> Result<(RespToken, &[u8]), Error> {
    if input.len() < 2 {
        bail!("Invalid bulk string format")
    }

    let (size, rest) = parse_until_eol(input)?;
    let (bulk, rest) = parse_until_eol(rest)?;
    let size: usize = std::str::from_utf8(size)?.parse()?;

    return Ok((RespToken::BulkString((bulk, size as u8)), rest));
}
