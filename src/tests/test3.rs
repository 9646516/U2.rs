use crate::u2client;

#[test]
fn main() {
    let f = std::fs::read_to_string("args.toml").unwrap();
    let args: u2client::types::Config = toml::from_str(f.as_str()).unwrap();
    println!("{:?}", args);
}
