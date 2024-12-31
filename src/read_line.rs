use std::path::Path;

use heapless::String;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug)]
pub enum ReadLineError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
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
    string
        .push_str(timings)
        .map_err(|()| ReadLineError::Capacity)?;

    Ok(string)
}

impl From<std::io::Error> for ReadLineError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
impl From<std::str::Utf8Error> for ReadLineError {
    fn from(err: std::str::Utf8Error) -> Self {
        Self::Utf8(err)
    }
}
impl std::fmt::Display for ReadLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Utf8(err) => write!(f, "UTF-8 error: {err}"),
            Self::Capacity => write!(f, "Capacity error"),
        }
    }
}

impl std::error::Error for ReadLineError {}
