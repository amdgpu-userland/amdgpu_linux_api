#[test]
fn opening_kfd() {
    let kfd = amdkfd::Kfd::open().expect("Please run this on linux with a modern AMD gpu");
    let version = kfd.version();
    println!("{version:?}");
    drop(kfd);
}
