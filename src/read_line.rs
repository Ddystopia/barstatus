use std::path::Path;

use heapless::String;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(thiserror::Error, Debug)]
pub enum ReadLineError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Capacity error")]
    Capacity,
}

pub async fn read_line_from_path<const N: usize>(
    path: impl AsRef<Path>,
) -> Result<String<N>, ReadLineError> {
    let mut file = File::open(path).await?;

    let mut timings = [0; N];
    let mut read = 0;
    while read < timings.len() {
        let bytes_read = file.read(&mut timings[read..]).await?;
        let newline = memchr::memrchr(b'\n', &timings[read..][..bytes_read]);
        let newline = newline.map(|pos| read + pos);
        read += bytes_read;
        if bytes_read == 0 {
            break;
        }
        if let Some(newline_position) = newline {
            read = newline_position;
            break;
        }
    }

    let timings = std::str::from_utf8(&timings[..read])?;
    let mut string = heapless::String::new();
    string.push_str(timings).map_err(|()| ReadLineError::Capacity)?;

    Ok(string)
}
