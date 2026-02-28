fn main() {
    let file1 = std::fs::File::open("/dev/dri/renderD128").unwrap();
    let file2 = std::fs::File::open("/dev/dri/card1").unwrap();
    let file3 = std::fs::File::open("/dev/dri/renderD128").unwrap();
    println!("You can check fdinfo and clients in debugfs, there should be 3 clients.");
    let _ = std::io::stdin().read_line(&mut String::new());
    drop(file1);
    drop(file3);

    println!("You can check fdinfo and clients in debugfs, there should be one primary client");
    let _ = std::io::stdin().read_line(&mut String::new());

    drop(file2);
}
