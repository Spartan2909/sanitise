# sanitise

A library for headache-free data clean-up and validation.

[![crates.io](https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust)](https://crates.io/crates/sanitise) 
[![github](https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/Spartan2909/rulox)
[![docs.rs](https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/sanitise/latest) 

`sanitise` is a CSV processing and validation library that generates code at compile time based on a YAML configuration file. The generated code is robust and will not panic.

`no_std` environments are supported, but the `alloc` crate is required.

## Quick Start

Add `sanitise` to your dependencies in your `Cargo.toml`:
```toml
[dependencies]
sanitise = "0.1"
```

Import the macro:
```rust
use sanitise::sanitise;
```

And call:
```rust,ignore
// main.rs
use std::{fs, iter::zip};

use sanitise::sanitise;

fn main() {
    let csv = fs::read_to_string("data.csv").unwrap();
    let ((time_millis, pulse, movement), (time_secs,)) = sanitise!(include_str!("sanitise_config.yaml"), &csv).unwrap();

    println!("time_millis,time_secs,pulse,movement");
    for (((time_millis, pulse), movement), time_secs) in zip(zip(zip(time_millis, pulse), movement), time_secs) {
        println!("{time_millis},{time_secs},{pulse},{movement}")
    }
}
```

```yaml
# sanitise_config.yaml
processes:
  - name: validate
    columns:
      - title: time
        type: integer
      - title: pulse
        type: integer
        max: 100
        min: 40
        on-invalid: average
        valid-streak: 3
      - title: movement
        type: integer
        valid-values: [0, 1]
        output-type: boolean
        output: "value == 1"
  - name: process
    columns:
      - title: time
        type: integer
        output: "value / 1000"
      - title: pulse
        type: integer
        ignore: true
      - title: movement
        type: integer
        ignore: true

```

```csv
# data.csv
time,pulse,movement
0,67,0
15,45,1
126,132,1
```

The first argument to `sanitise!` must be either a string literal or a macro call that expands to a string literal. The second argument must be an expression that resolves to a `&str` in CSV format. In the above example, `sanitise_config.yaml` must be next to `main.rs`, and `data.csv` must be in the working directory at runtime.

## Configuration

For details on the configuration file, see the [specification](https://github.com/Spartan2909/sanitise/blob/main/configuration.md).

## Optional features

- `benchmark`: Print the time taken to complete various stages of the process. Disables `no_std` support. You probably don't want this.

## Efficiency
The macro creates linear finite automata to process each column. If `on-invalid` is set to `average` for a given column, that column's automaton will use a state machine to keep track of valid and invalid values. If a column is ignored, no automaton will be generated for it. All data is stored in native Rust types.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
