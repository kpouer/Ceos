use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];

pub(crate) fn is_gzip(buffer: &mut (impl Read + Seek)) -> bool {
    let mut buf = [0; 2];
    let result = match buffer.read_exact(&mut buf) {
        Ok(_) => buf == GZIP_MAGIC,
        Err(_) => false,
    };
    let _ = buffer.seek(SeekFrom::Start(0));
    result
}

pub(crate) fn gzip_uncompressed_size_fast(path: &std::path::Path) -> std::io::Result<u32> {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::End(-4))?;
    let mut buf = [0u8; 4];
    f.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf)) // ISIZE
}
