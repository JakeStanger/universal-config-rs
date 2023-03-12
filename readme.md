# Universal Config

[crate](https://crates.io/crates/universal-config) | 
[docs](https://docs.rs/universal-config) | 
[repo](https://github.com/jakestanger/universal-config-rs)

Universal Config is a library for Rust to simplify *reading* configuration files.
It is able to automatically locate config files from standard locations, and has support for various file formats.

The crate does not offer a lot of functionality, leaving that to Serde and your implementation. 
Instead, it just deals with loading the file.

Currently, the following formats are supported:

- JSON
- YAML
- TOML
- [Corn](https://github.com/jakestanger/corn)

# Installation

Just add the crate:

```
cargo add universal-config
```

By default, support for all languages is included. 
You can enable/disable languages using feature flags. For example, in your `Cargo.toml`:

```toml
[dependencies.universal-config]
version = "0.1.0"
default-features = false
features = ["json", "toml"]
```

## Example usage

```rust
use universal_config::Config;
use serde::Deserialize;

#[derive(Deserialize)]
struct MyConfig {
    foo: String,
}

fn main() {
    let config: MyConfig = Config::new("my-app").find_and_load();
    println!("{}", config.foo);
}
```