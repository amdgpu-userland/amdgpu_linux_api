use std::ffi::CString;

fn main() {
    let path = CString::new("/dev/dri/renderD128").unwrap();

    let base_modes = [
        (libc::O_RDONLY, "O_RDONLY"),
        (libc::O_WRONLY, "O_WRONLY"),
        (libc::O_RDWR, "O_RDWR"),
    ];

    let modifiers = [
        (libc::O_APPEND, "O_APPEND"),
        (libc::O_ASYNC, "O_ASYNC"),
        (libc::O_CLOEXEC, "O_CLOEXEC"),
        (libc::O_CREAT, "O_CREAT"),
        (libc::O_DIRECT, "O_DIRECT"),
        (libc::O_DIRECTORY, "O_DIRECTORY"),
        (libc::O_DSYNC, "O_DSYNC"),
        (libc::O_EXCL, "O_EXCL"),
        (libc::O_LARGEFILE, "O_LARGEFILE"),
        (libc::O_NOATIME, "O_NOATIME"),
        (libc::O_NOCTTY, "O_NOCTTY"),
        (libc::O_NOFOLLOW, "O_NOFOLLOW"),
        (libc::O_NONBLOCK, "O_NONBLOCK"),
        (libc::O_PATH, "O_PATH"),
        (libc::O_SYNC, "O_SYNC"),
        (libc::O_TMPFILE, "O_TMPFILE"),
        (libc::O_TRUNC, "O_TRUNC"),
    ];

    println!("Testing flags for render node: /dev/dri/renderD128");

    // First test single base modes
    for &(mode, mode_name) in &base_modes {
        test_open(&path, mode, mode_name);

        // Test base mode + single modifier
        for &(modifier, modifier_name) in &modifiers {
            let combined = mode | modifier;
            let name = format!("{} | {}", mode_name, modifier_name);
            test_open(&path, combined, &name);
        }
    }
}

fn test_open(path: &CString, flags: libc::c_int, name: &str) {
    let fd = unsafe { libc::open(path.as_ptr(), flags, 0o666) };
    if fd >= 0 {
        println!("SUCCESS: {}", name);
        unsafe { libc::close(fd) };
    } else {
        let err = std::io::Error::last_os_error();
        println!("FAILURE: {} ({:?})", name, err);
    }
}
