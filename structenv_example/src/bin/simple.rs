#[macro_use]
extern crate structenv_derive;
extern crate structenv;

use std::net::IpAddr;

#[derive(StructEnv, Debug)]
struct Env {
    foo: bool,
    #[structenv(default_value = r#""bar".to_string()"#)]
    bar: String,
    host_address: IpAddr,
}

fn main() {
    use structenv::StructEnv;

    let env = Env::from_env();
    println!("{:?}", env);
}
