use core::fmt;

use anyhow::{anyhow, Error};

pub const CR: u8 = b'\r';
pub const LF: u8 = b'\n';

#[derive(Debug)]
pub enum RespToken<'a> {
    Error(&'a [u8]),
    Integer(&'a [u8]),
    SimpleString(&'a [u8]),
    BulkString((&'a [u8], u8)),
    Array((Vec<RespToken<'a>>, u8)),
}

impl<'a> RespToken<'a> {
    pub fn unpack(&self) -> &[u8] {
        match *self {
            RespToken::Error(val) => val,
            RespToken::Integer(val) => val,
            RespToken::SimpleString(val) => val,
            RespToken::BulkString((val, _)) => val,
            _ => panic!("Can't unpack array"), // TODO: probably not a good interface
        }
    }

    pub fn deserialize(input: &'a [u8]) -> Result<Self, anyhow::Error> {
        let (res, rem) = parse_input(input)?;
        if rem.len() > 0 {
            return Err(anyhow!("Invalid input: {}", String::from_utf8_lossy(rem)));
        }
        Ok(res)
    }
}

impl fmt::Display for RespToken<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespToken::Error(val) => write!(f, "-{}", String::from_utf8_lossy(val)),
            RespToken::Integer(val) => write!(f, ":{}", String::from_utf8_lossy(val)),
            RespToken::SimpleString(val) => write!(f, "+{}", String::from_utf8_lossy(val)),
            RespToken::BulkString((val, len)) => {
                write!(f, "${}{}", len, String::from_utf8_lossy(val))
            }
            RespToken::Array((val, _)) => {
                let mut res = String::new();
                for el in val {
                    res.push_str(&format!("{}\r\n", el));
                }
                write!(f, "*{}", res)
            }
        }
    }
}

pub fn parse_input(input: &[u8]) -> Result<(RespToken, &[u8]), anyhow::Error> {
    match input[0] {
        b'+' => parse_simple_string(&input[1..]),
        b'$' => parse_bulk_string(&input[1..]),
        b'*' => parse_array(&input[1..]),
        b':' => parse_integer(&input[1..]),
        byte => panic!(
            "Invalid type specifier {}, must be valid RESP protocol",
            byte as char
        ),
    }
}

pub fn parse_until_eol(input: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    for (idx, el) in input.iter().enumerate() {
        if el == &CR {
            if input[idx + 1] != LF {
                return Err(anyhow!("Invalid EOL format"));
            }
            return Ok((&input[..idx], &input[idx + 2..]));
        }
    }
    Err(anyhow!("not enough bytes"))
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
        return Err(anyhow!("Invalid simple string format"));
    }

    let (data, rest) = parse_until_eol(input)?;
    return Ok((RespToken::SimpleString(data), rest));
}

pub fn parse_bulk_string(input: &[u8]) -> Result<(RespToken, &[u8]), Error> {
    if input.len() < 2 {
        return Err(anyhow!("Invalid bulk string format"));
    }

    let (size, rest) = parse_until_eol(input)?;
    let (bulk, rest) = parse_until_eol(rest)?;
    let size: usize = std::str::from_utf8(size)?.parse()?;

    return Ok((RespToken::BulkString((bulk, size as u8)), rest));
}

pub fn parse_integer(input: &[u8]) -> Result<(RespToken, &[u8]), Error> {
    if input.len() < 2 {
        return Err(anyhow!("Invalid integer format"));
    }

    let (integer, rest) = parse_until_eol(input)?;

    //INFO: for now we don't care about the integer value, just storing the raw bytes
    return Ok((RespToken::Integer(integer), rest));
}

pub fn parse_error(input: &[u8]) -> Result<(RespToken, &[u8]), Error> {
    todo!()
}
