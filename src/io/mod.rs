/// Module public `block`
pub mod block;
/// Module public `decompress`
pub mod decompress;
/// Module public `download`
pub mod download;

use std::io::Read;

/// Read until buf is full or EOF.
pub fn read_full(reader: &mut dyn Read, buf: &mut [u8]) -> std::io::Result<usize> {
    let mut total = 0;
    while total < buf.len() {
        let n = reader.read(&mut buf[total..])?;
        if n == 0 {
            break;
        }
        total += n;
    }
    Ok(total)
}
