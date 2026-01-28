use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub(crate) fn gzip_uncompressed_size_fast(path: &std::path::Path) -> std::io::Result<u32> {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::End(-4))?;
    let mut buf = [0u8; 4];
    f.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf)) // ISIZE
}