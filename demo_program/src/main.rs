use base64;
fn main() {
    println!(
        "{}",
        std::str::from_utf8(
            &base64::decode(std::str::from_utf8(include_bytes!("test.txt")).unwrap()).unwrap()
        )
        .unwrap()
    );
    std::thread::sleep(std::time::Duration::from_millis(10000));
}
