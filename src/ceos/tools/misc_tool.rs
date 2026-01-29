use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];

pub(crate) fn is_gzip<R>(reader: &mut R) -> bool
where
    R: Read + Seek,
{
    let mut magic = [0; GZIP_MAGIC.len()];
    let start_pos = reader.stream_position().ok().unwrap_or(0);
    let is_gzip = if reader.read_exact(&mut magic).is_ok() {
        magic == GZIP_MAGIC
    } else {
        false
    };
    let _ = reader.seek(SeekFrom::Start(start_pos));
    is_gzip
}

pub(crate) fn gzip_uncompressed_size_fast(path: &std::path::Path) -> std::io::Result<u32> {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::End(-4))?;
    let mut buf = [0u8; 4];
    f.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf)) // ISIZE
}
