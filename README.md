# Universal Config

[crate](https://crates.io/crates/universal-config) | 
[docs](https://docs.rs/universal-config) | 
[repo](https://github.com/jakestanger/universal-config-rs)

Universal Config is a library for Rust to simplify reading and writing configuration files.
It is able to automatically locate config files from standard locations, and has support for various file formats.

The crate does not offer a lot of functionality, leaving that to Serde and your implementation. 
Instead, it just deals with loading and saving the file.

Currently, the following formats are supported:

- JSON via `serde_json`
- YAML via `serde_yaml`
- TOML via `toml`
- XML via `serde_xml_rs`
- [Corn](https://github.com/jakestanger/corn) via `libcorn`

# Installation

Just add the crate:

```bash
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

```no_run
use universal_config::ConfigLoader;
use serde::Deserialize;

#[derive(Deserialize)]
struct MyConfig {
    foo: String,
}

fn main() {
    let config: MyConfig = ConfigLoader::new("my-app").find_and_load().unwrap();
    println!("{}", config.foo);
}
```

For more advanced usage, please check the [docs](https://docs.rs/universal-config).