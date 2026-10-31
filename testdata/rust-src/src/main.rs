fn greet(name: &str) -> String {
    format!("hello, {name}")
}

fn main() {
    let msg = greet("rust-src");
    let ok = msg == "hello, rust-src";
    println!(r#"{{"msg":"{msg}","ok":{ok}}}"#);
    if !ok {
        std::process::exit(1);
    }
}
