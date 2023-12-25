use anyhow::Error;

pub fn _utf8_to_string(input: &[u8]) -> Result<String, Error> {
    let s = std::str::from_utf8(input)?;
    Ok(s.to_string())
}
