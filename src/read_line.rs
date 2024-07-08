use std::{fs::File, io::Read, path::Path};

use anyhow::Context;
use heapless::String;

pub fn read_line_from_path<const N: usize>(path: impl AsRef<Path>) -> anyhow::Result<String<N>> {
    let mut file = File::open(path).context("Failed to open file")?;

    let mut timings = [0; N];
    let mut read = 0;
    while read < timings.len() {
        let bytes_read = file.read(&mut timings).context("Failed to read file")?;
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

    let timings = std::str::from_utf8(&timings[..read]).context("Non-UTF8 data")?;
    let mut string = heapless::String::new();
    string
        .push_str(timings)
        .map_err(|()| anyhow::anyhow!("Capacity exceeded while reading line from file"))?;

    Ok(string)
}
