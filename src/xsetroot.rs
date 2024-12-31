use std::ffi::CStr;

#[cfg(not(feature = "xsetroot_dyn"))]
pub fn set_on_bar(val: &CStr) -> Result<(), std::io::Error> {
    let val = val.to_str().unwrap();
    let status = std::process::Command::new("xsetroot")
        .args(["-name", val])
        .spawn()?
        .wait()?;

    match status.code() {
        Some(0) => Ok(status),
        Some(code) => anyhow::bail!("xsetroot exited with code {code}"),
        None => anyhow::bail!("xsetroot exited by signal"),
    }
}

#[cfg(feature = "xsetroot_dyn")]
pub fn set_on_bar(val: &CStr) -> anyhow::Result<()> {
    // SAFETY: This is more or less direct rewrite from `xsetroot.c`
    unsafe {
        let xlib = x11_dl::xlib::Xlib::open().unwrap();

        let dpy = (xlib.XOpenDisplay)(std::ptr::null());
        if dpy.is_null() {
            anyhow::bail!("XOpenDisplay failed");
        }
        let screen = (xlib.XDefaultScreen)(dpy);
        let root = (xlib.XRootWindow)(dpy, screen);

        (xlib.XStoreName)(dpy, root, val.as_ptr());
        (xlib.XCloseDisplay)(dpy);
    };

    Ok(())
}
