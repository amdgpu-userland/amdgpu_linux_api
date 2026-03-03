fn main() {
    let file = std::fs::File::open("/dev/kfd").unwrap();
    drop(file);

    let file = std::fs::File::open("/dev/kfd").unwrap();

    println!("Check dmesg, it should show kfd process already found");
    let _ = std::io::stdin().read_line(&mut String::new());

    drop(file);
}
