use std::io::Write;

#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "xsetroot_dyn")]
    XOpendisplayFailed,
    #[cfg(feature = "xsetroot_dyn")]
    FailedToOpenXlib(x11_dl::error::OpenError),
    #[cfg(not(feature = "xsetroot_dyn"))]
    NotUtf8(std::str::Utf8Error),
    #[cfg(not(feature = "xsetroot_dyn"))]
    Io(std::io::Error),
    #[cfg(not(feature = "xsetroot_dyn"))]
    XSetRootCode(i32),
    #[cfg(not(feature = "xsetroot_dyn"))]
    XSetRootSignal,
}

#[cfg(not(feature = "xsetroot_dyn"))]
pub fn set_on_bar(line: &str) -> Result<(), Error> {
    let mut buf: [u8; 256] = [0; 256];
    let mut writer = std::io::Cursor::new(&mut buf[..]);
    if let Err(err) = write!(writer, "{line: >93}") {
        unreachable!("Buffer is big enough for the line: {err}");
    }
    let position = writer.position() as usize + 1;
    let line = std::str::from_utf8(&buf[..position])?;

    let status = std::process::Command::new("xsetroot")
        .args(["-name", line])
        .spawn()?
        .wait()?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(Error::XSetRootCode(code)),
        None => Err(Error::XSetRootSignal),
    }
}

#[cfg(feature = "xsetroot_dyn")]
pub fn set_on_bar(line: &str) -> Result<(), Error> {
    let mut buf: [u8; 256] = [0; 256];
    let mut writer = std::io::Cursor::new(&mut buf[..]);
    if let Err(err) = write!(writer, "{line: >93}") {
        unreachable!("Error while writing to alignment buffer: {err}");
    }
    let position = writer.position() as usize + 1;
    let line = std::ffi::CStr::from_bytes_with_nul(&buf[..position]);
    let line = line.expect("We have zero byte at the end for sure");

    // SAFETY: This is more or less direct rewrite from `xsetroot.c`
    unsafe {
        let xlib = x11_dl::xlib::Xlib::open()?;

        let dpy = (xlib.XOpenDisplay)(std::ptr::null());
        if dpy.is_null() {
            return Err(Error::XOpendisplayFailed);
        }
        let screen = (xlib.XDefaultScreen)(dpy);
        let root = (xlib.XRootWindow)(dpy, screen);

        (xlib.XStoreName)(dpy, root, line.as_ptr());
        (xlib.XCloseDisplay)(dpy);
    };

    Ok(())
}

#[cfg(not(feature = "xsetroot_dyn"))]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

#[cfg(not(feature = "xsetroot_dyn"))]
impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::NotUtf8(err)
    }
}

#[cfg(feature = "xsetroot_dyn")]
impl From<x11_dl::error::OpenError> for Error {
    fn from(value: x11_dl::error::OpenError) -> Self {
        Self::FailedToOpenXlib(value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "xsetroot_dyn")]
            Self::XOpendisplayFailed => write!(f, "Failed to open display"),
            #[cfg(feature = "xsetroot_dyn")]
            Self::FailedToOpenXlib(err) => write!(f, "Failed to open xlib: {err}"),
            #[cfg(not(feature = "xsetroot_dyn"))]
            Self::Io(err) => write!(f, "IO error: {err}"),
            #[cfg(not(feature = "xsetroot_dyn"))]
            Self::XSetRootCode(code) => write!(f, "xsetroot exited with code: {code}"),
            #[cfg(not(feature = "xsetroot_dyn"))]
            Self::XSetRootSignal => write!(f, "xsetroot was killed by a signal"),
            #[cfg(not(feature = "xsetroot_dyn"))]
            Self::NotUtf8(err) => write!(f, "Not UTF-8: {err}"),
        }
    }
}

impl std::error::Error for Error {}
