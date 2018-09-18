# StructEnv

initialize structs from envrionment variables

``` rust
#[macro_use]
extern crate structenv_derive;

#[derive(StructEnv, Debug)]
struct Env {
    foo: bool,
    #[structenv(default_value = r#""bar".to_string()"#)]
    bar: String,
    host_address: IpAddr,
}

fn main() {
    // `from_env` is generated
    let env = Env::from_env();
    println!("{:?}", env);
}
```

``` console
$ export FOO=false
$ export HOST_ADDRESS=127.0.0.2
$ cargo run -p structenv_example --bin simple
Env { foo: false, bar: "bar", host_address: V4(127.0.0.2) }
```

