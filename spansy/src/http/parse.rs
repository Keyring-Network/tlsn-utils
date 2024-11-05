use crate::ParseError;
use bytes::{Bytes, BytesMut};
use utils::range::RangeSet;

// Parsing functions for Transfer-Encoding header types
/// Parse Transfer-Encoding: chunked body
pub fn parse_chunked_body(src: &Bytes, offset: usize) -> Result<(Bytes, RangeSet<usize>, usize), ParseError> {
    let mut body = BytesMut::new();
    let mut pos = offset;
    let mut ranges = Vec::new();

    loop {
        let chunk_size_end = src[pos..]
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or_else(|| ParseError("Invalid chunked encoding: missing chunk size CRLF".to_string()))?
            + pos;
        
        let chunk_size_str = std::str::from_utf8(&src[pos..chunk_size_end])
            .map_err(|_| ParseError("Invalid chunk size encoding".to_string()))?;
        let chunk_size = usize::from_str_radix(chunk_size_str.trim(), 16)
            .map_err(|_| ParseError("Invalid chunk size value".to_string()))?;
        
        let chunk_start = chunk_size_end + 2;
        
        if chunk_size == 0 {
            break;
        }
        
        let chunk_end = chunk_start + chunk_size;
        if chunk_end > src.len() {
            return Err(ParseError("Chunk data exceeds source length".to_string()));
        }

        ranges.push(chunk_start..chunk_end);
        body.extend_from_slice(&src[chunk_start..chunk_end]);
        
        pos = chunk_end + 2;
    }

    pos += 2;
    
    Ok((body.freeze(), RangeSet::from(ranges), pos))
}
