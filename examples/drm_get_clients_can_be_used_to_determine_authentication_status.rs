use std::os::fd::AsRawFd;

use amdgpu_linux_api::drm::ioctl::drm::{Client, drop_master, get_client, set_master};

fn main() {
    println!("Run this test when there is no drm_master for default device.");
    println!("For example use `chvt 3` or CTRL+ALT+3");

    let file = std::fs::File::open("/dev/dri/card1").unwrap();

    let _ = unsafe { set_master(file.as_raw_fd()) };

    println!("You can check if it got master in debugfs clients file");
    let _ = std::io::stdin().read_line(&mut String::new());

    let mut args = Client {
        idx: 0,
        ..Default::default()
    };
    let res = unsafe { get_client(file.as_raw_fd(), &mut args) };
    assert!(matches!(res, Ok(_)));
    assert_eq!(
        args.auth, 1,
        "Master client should be automatically authenticated"
    );

    let _ = unsafe { drop_master(file.as_raw_fd()) };

    let mut args = Client {
        idx: 0,
        ..Default::default()
    };
    let res = unsafe { get_client(file.as_raw_fd(), &mut args) };
    assert!(matches!(res, Ok(_)));
    assert_eq!(
        args.auth, 1,
        "After droping master, the primary client should still be authenticated"
    );
}
