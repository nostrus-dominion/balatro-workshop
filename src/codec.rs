use crate::{
    lua::{self, Value, MAX_TEXT},
    Result,
};
use flate2::{write::DeflateEncoder, Compression, Decompress, FlushDecompress, Status};
use std::io::Write;

pub const MAX_COMPRESSED: usize = 16 * 1024 * 1024;

pub fn decode(bytes: &[u8]) -> Result<Value> {
    if bytes.len() > MAX_COMPRESSED {
        return Err("Compressed save exceeds 16 MiB.".into());
    }
    let mut decoder = Decompress::new(false);
    let mut out = Vec::new();
    loop {
        let before_in = decoder.total_in();
        let before_out = decoder.total_out();
        let mut block = [0u8; 8192];
        let status = decoder
            .decompress(
                &bytes[before_in as usize..],
                &mut block,
                FlushDecompress::None,
            )
            .map_err(|e| format!("Invalid raw-DEFLATE .jkr file: {e}"))?;
        let written = (decoder.total_out() - before_out) as usize;
        if out.len() + written > MAX_TEXT {
            return Err("Expanded save exceeds 32 MiB.".into());
        }
        out.extend_from_slice(&block[..written]);
        if status == Status::StreamEnd {
            if decoder.total_in() as usize != bytes.len() {
                return Err("Unexpected data after compressed save.".into());
            }
            return lua::parse(&out);
        }
        if before_in == decoder.total_in() && before_out == decoder.total_out() {
            return Err("Truncated or incomplete compressed save.".into());
        }
    }
}

pub fn encode(value: &Value) -> Result<Vec<u8>> {
    let text = lua::serialize(value);
    if lua::parse(text.as_bytes())? != *value {
        return Err("Save failed round-trip validation.".into());
    }
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(text.as_bytes())
        .map_err(|e| e.to_string())?;
    let bytes = encoder.finish().map_err(|e| e.to_string())?;
    if decode(&bytes)? != *value {
        return Err("Compressed save failed validation.".into());
    }
    Ok(bytes)
}
